//! Raw USB LK unlock and raw getvars.
//!
//! The LK fastboot CLI has a reconnect bug. All LK interaction here goes over
//! libusb, so there is no fastboot process to wedge.

use crate::consts::{PAYLOAD, SUPPORTED_LK};
use crate::device;
use crate::report::Reporter;
use crate::{Result, Settings};
use rusb::{Context, DeviceHandle, Direction, TransferType, UsbContext};
use std::time::{Duration, Instant};

const VID: u16 = 0x0e8d;
const PID: u16 = 0x201c;

#[derive(Clone, Debug)]
pub struct LkInfo {
    pub version: String,
    pub product: String,
    pub slot: String,
    pub unlocked: bool,
}

#[derive(Clone, Debug)]
pub enum Outcome {
    Unlocked(LkInfo),
    AlreadyUnlocked(LkInfo),
}

struct Session {
    handle: DeviceHandle<Context>,
    iface: u8,
    ep_in: u8,
    ep_out: u8,
}

fn find_lk_fastboot() -> Result<rusb::Device<Context>> {
    let ctx = Context::new().map_err(|e| anyhow::anyhow!("libusb init failed: {e}"))?;
    let devices = ctx
        .devices()
        .map_err(|e| anyhow::anyhow!("libusb list failed: {e}"))?;
    for device in devices.iter() {
        let desc = device
            .device_descriptor()
            .map_err(|e| anyhow::anyhow!("usb descriptor failed: {e}"))?;
        if desc.vendor_id() == VID && desc.product_id() == PID {
            return Ok(device);
        }
    }
    anyhow::bail!("no 0e8d:201c LK fastboot device found")
}

pub fn lk_present() -> bool {
    find_lk_fastboot().is_ok()
}

fn open_session(device: &rusb::Device<Context>) -> Result<Session> {
    let config = device
        .active_config_descriptor()
        .map_err(|e| anyhow::anyhow!("usb config failed: {e}"))?;
    let mut found = None;
    for iface in config.interfaces() {
        for desc in iface.descriptors() {
            let mut ep_in = None;
            let mut ep_out = None;
            for ep in desc.endpoint_descriptors() {
                if ep.transfer_type() == TransferType::Bulk {
                    match ep.direction() {
                        Direction::In => ep_in = Some(ep.address()),
                        Direction::Out => ep_out = Some(ep.address()),
                    }
                }
            }
            if let (Some(i), Some(o)) = (ep_in, ep_out) {
                found = Some((iface.number(), i, o));
            }
        }
    }
    let (iface, ep_in, ep_out) = found.ok_or_else(|| anyhow::anyhow!("no bulk endpoints"))?;
    let handle = device
        .open()
        .map_err(|e| anyhow::anyhow!("usb open failed: {e}"))?;
    let _ = handle.set_auto_detach_kernel_driver(true);
    handle
        .claim_interface(iface)
        .map_err(|e| anyhow::anyhow!("usb claim interface failed: {e}"))?;
    Ok(Session {
        handle,
        iface,
        ep_in,
        ep_out,
    })
}

fn command(rep: &dyn Reporter, s: &Session, data: &[u8], timeout_ms: u64) -> Result<Vec<u8>> {
    let timeout = Duration::from_millis(timeout_ms);
    let n = s
        .handle
        .write_bulk(s.ep_out, data, timeout)
        .map_err(|e| anyhow::anyhow!("usb write failed: {e}"))?;
    if n != data.len() {
        anyhow::bail!("short USB write: {n}/{}", data.len());
    }
    let mut buf = [0u8; 512];
    let n = s
        .handle
        .read_bulk(s.ep_in, &mut buf, timeout)
        .map_err(|e| anyhow::anyhow!("usb read failed: {e}"))?;
    rep.log(&format!(
        "raw USB iface {} IN {:#04x} OUT {:#04x}",
        s.iface, s.ep_in, s.ep_out
    ));
    Ok(buf[..n].to_vec())
}

fn getvar(rep: &dyn Reporter, s: &Session, name: &str) -> Result<String> {
    let mut cmd = format!("getvar:{name}").into_bytes();
    cmd.push(0);
    let resp = command(rep, s, &cmd, 3500)?;
    let text = String::from_utf8_lossy(&resp)
        .trim_end_matches('\0')
        .to_string();
    if let Some(value) = text.strip_prefix("OKAY") {
        return Ok(value.trim().to_string());
    }
    if let Some(err) = text.strip_prefix("FAIL") {
        anyhow::bail!("getvar:{name} failed: {}", err.trim());
    }
    Ok(text.trim().to_string())
}

fn read_info(rep: &dyn Reporter, s: &Session) -> Result<LkInfo> {
    let version = getvar(rep, s, "version-bootloader")?;
    let product = getvar(rep, s, "product")?;
    let slot = getvar(rep, s, "current-slot")?;
    let unlocked = getvar(rep, s, "unlocked")?;
    let userspace = getvar(rep, s, "is-userspace")?;
    rep.log(&format!("LK version: {version}"));
    rep.log(&format!("LK product: {product}"));
    rep.log(&format!(
        "active slot: {}",
        if slot.is_empty() { "?" } else { &slot }
    ));
    rep.log(&format!("unlocked: {unlocked}"));
    rep.log(&format!("is-userspace: {userspace}"));
    if userspace.eq_ignore_ascii_case("yes") {
        anyhow::bail!("this is fastbootd, not LK fastboot");
    }
    if !product.is_empty() && product != "T405A" {
        anyhow::bail!("LK product is {product:?}, expected T405A");
    }
    if !device::supported_lk(&version) {
        let supported = SUPPORTED_LK.iter().map(|(v, _)| *v).collect::<Vec<_>>();
        anyhow::bail!("bootloader {version:?} is not in the verified list {supported:?}");
    }
    Ok(LkInfo {
        version,
        product,
        slot,
        unlocked: unlocked.eq_ignore_ascii_case("yes"),
    })
}

fn send_payload(rep: &dyn Reporter, s: &Session) -> Result<()> {
    rep.log(&format!("payload ({} bytes)", PAYLOAD.len() + 1));
    rep.log("sending exactly once");
    let mut cmd = PAYLOAD.to_vec();
    cmd.push(0);
    match command(rep, s, &cmd, 5500) {
        Ok(resp) => rep.log(&format!("response = {:?}", String::from_utf8_lossy(&resp))),
        Err(e) => rep.log(&format!(
            "USB error after send (expected if the tablet rebooted): {e}"
        )),
    }
    rep.log("SENT ONCE. Do not resend. Check the tablet for the orange state.");
    Ok(())
}

pub fn read_lk_info(rep: &dyn Reporter, timeout_secs: u64) -> Result<LkInfo> {
    let session = wait_for_lk(rep, timeout_secs)?;
    read_info(rep, &session)
}

fn wait_for_lk(rep: &dyn Reporter, timeout_secs: u64) -> Result<Session> {
    let deadline = Instant::now() + Duration::from_secs(timeout_secs);
    let mut last = false;
    while Instant::now() < deadline {
        if let Ok(device) = find_lk_fastboot() {
            return open_session(&device);
        }
        if !last {
            rep.log("waiting for LK fastboot over USB...");
            last = true;
        }
        std::thread::sleep(Duration::from_millis(250));
    }
    anyhow::bail!("timed out after {timeout_secs}s waiting for LK fastboot")
}

/// Wait for LK, read info, and send the payload if still locked.
pub fn wait_and_unlock(rep: &dyn Reporter, timeout_secs: u64) -> Result<Outcome> {
    let session = wait_for_lk(rep, timeout_secs)?;
    let info = read_info(rep, &session)?;
    if info.unlocked {
        rep.log("already unlocked; not sending the payload");
        rep.log("requesting fastbootd over raw USB");
        let mut cmd = b"reboot-fastboot".to_vec();
        cmd.push(0);
        let _ = command(rep, &session, &cmd, 3000);
        return Ok(Outcome::AlreadyUnlocked(info));
    }
    send_payload(rep, &session)?;
    Ok(Outcome::Unlocked(info))
}

/// Advanced unlock command: requires the tablet to be booted in Android or
/// recovery with adb. The wizard uses wait_and_unlock directly.
pub fn unlock(rep: &dyn Reporter, _settings: &Settings) -> Result<()> {
    let summary = device::require_model_any(rep)?;
    if summary.locked == "0" {
        anyhow::bail!("device is already unlocked (ro.boot.flash.locked=0); nothing to do");
    }
    let trigger_rep = crate::StdoutReporter;
    let handle = std::thread::spawn(move || {
        let _ = crate::proc::adb_ok(&trigger_rep, &["reboot", "bootloader"]);
    });
    let outcome = wait_and_unlock(rep, 180)?;
    let _ = handle.join();
    match outcome {
        Outcome::AlreadyUnlocked(info) => rep.log(&format!(
            "already unlocked on slot {} (LK {})",
            info.slot, info.version
        )),
        Outcome::Unlocked(info) => rep.log(&format!(
            "payload sent on slot {} (LK {}); device will reboot",
            info.slot, info.version
        )),
    }
    rep.log("waiting for recovery or Android adb");
    device::wait_for_adb(rep, 900, false)?;
    let now = device::summary(rep)?;
    if now.locked != "0" {
        anyhow::bail!("still locked after the payload; do NOT resend - inspect the tablet");
    }
    rep.log(&format!(
        "unlocked: flash.locked={} verifiedbootstate={}",
        now.locked, now.verified_boot
    ));
    Ok(())
}
