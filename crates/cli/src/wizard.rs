//! Single-command guided install wizard.

use onn11core_core as core;
use onn11core_core::{
    device, download, flash, magisk, unlock, Reporter, Settings, StdoutReporter, Variant,
};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

enum State {
    Recovery(core::DeviceSummary),
    Android(core::DeviceSummary),
    Fastbootd,
    Lk,
    Sideload,
    Unauthorized,
    Unknown,
    None,
}

fn ask(prompt: &str, default: &str) -> String {
    print!("{prompt} [{default}]: ");
    let _ = io::stdout().flush();
    let mut line = String::new();
    let _ = io::stdin().read_line(&mut line);
    let value = line.trim();
    if value.is_empty() {
        default.to_string()
    } else {
        value.to_string()
    }
}

fn ask_yes(prompt: &str, default: bool) -> bool {
    let d = if default { "Y/n" } else { "y/N" };
    let value = ask(prompt, d);
    match value.to_lowercase().as_str() {
        "y" | "yes" => true,
        "n" | "no" => false,
        _ => default,
    }
}

fn slot_name(summary: &core::DeviceSummary) -> String {
    summary.slot.trim_start_matches('_').to_string()
}

fn detect(rep: &StdoutReporter) -> anyhow::Result<State> {
    match device::adb_mode() {
        device::AdbMode::Recovery => return Ok(State::Recovery(device::require_model_any(rep)?)),
        device::AdbMode::Android => return Ok(State::Android(device::require_model(rep)?)),
        device::AdbMode::Sideload => return Ok(State::Sideload),
        device::AdbMode::Unauthorized => return Ok(State::Unauthorized),
        device::AdbMode::Unknown => return Ok(State::Unknown),
        device::AdbMode::None => {}
    }
    if device::fastboot_serial().is_some() {
        if unlock::lk_present() {
            return Ok(State::Lk);
        }
        return Ok(State::Fastbootd);
    }
    if unlock::lk_present() {
        return Ok(State::Lk);
    }
    Ok(State::None)
}

fn wait_state(rep: &StdoutReporter, first: bool) -> anyhow::Result<State> {
    let mut announced = !first;
    loop {
        let state = detect(rep)?;
        match state {
            State::None => {
                if !announced {
                    rep.log("");
                    rep.log("Power off the tablet, hold Volume Up, and plug in USB.");
                    rep.log("This boots stock recovery with adb. Waiting...");
                    rep.log("Android with USB debugging and the Volume Up + Volume Down keybind also work.");
                    announced = true;
                }
            }
            State::Sideload => {
                if !announced {
                    rep.log("Recovery is in sideload mode. Reboot recovery normally, then wait.");
                    announced = true;
                }
            }
            State::Unauthorized => {
                if !announced {
                    rep.log("Approve the USB debugging prompt on the tablet.");
                    announced = true;
                }
            }
            State::Unknown => {
                if !announced {
                    rep.log("Waiting for Android or recovery adb to settle...");
                    announced = true;
                }
            }
            other => return Ok(other),
        }
        thread::sleep(Duration::from_secs(2));
    }
}

fn unlock_from_adb(rep: &StdoutReporter, summary: &core::DeviceSummary) -> anyhow::Result<()> {
    if summary.locked == "0" {
        rep.log("bootloader already unlocked");
        return Ok(());
    }
    rep.log("");
    rep.log("The bootloader is locked. The unlock wipes userdata and is irreversible.");
    let answer = ask("Type UNLOCK ONN11CORE to continue", "");
    if answer != "UNLOCK ONN11CORE" {
        anyhow::bail!("unlock not confirmed");
    }

    let handle = thread::spawn(|| {
        let rep = StdoutReporter;
        let _ = core::proc::adb_ok(&rep, &["reboot", "bootloader"]);
    });
    let outcome = unlock::wait_and_unlock(rep, 180)?;
    let _ = handle.join();
    match &outcome {
        unlock::Outcome::Unlocked(info) => {
            rep.log(&format!(
                "payload sent on LK {} slot {}",
                info.version, info.slot
            ));
            rep.log("waiting for recovery adb");
            device::wait_for_adb(rep, 900, false)?;
            let now = device::summary(rep)?;
            if now.locked != "0" {
                anyhow::bail!("still locked after the payload; do NOT resend");
            }
        }
        unlock::Outcome::AlreadyUnlocked(info) => {
            rep.log(&format!("already unlocked on slot {}", info.slot));
        }
    }
    Ok(())
}

fn unlock_from_lk(rep: &StdoutReporter) -> anyhow::Result<()> {
    rep.log("");
    rep.log("LK fastboot detected. Sending the unlock over raw USB.");
    let outcome = unlock::wait_and_unlock(rep, 60)?;
    match &outcome {
        unlock::Outcome::Unlocked(info) => {
            rep.log(&format!(
                "payload sent on LK {} slot {}",
                info.version, info.slot
            ));
            rep.log("waiting for recovery adb");
            device::wait_for_adb(rep, 900, false)?;
            let now = device::summary(rep)?;
            if now.locked != "0" {
                anyhow::bail!("still locked after the payload; do NOT resend");
            }
        }
        unlock::Outcome::AlreadyUnlocked(info) => {
            rep.log(&format!("already unlocked on slot {}", info.slot));
            rep.log("requested fastbootd; waiting...");
            device::wait_for_fastbootd(rep, 120)?;
        }
    }
    Ok(())
}

fn ensure_stock(rep: &StdoutReporter, settings: &Settings) -> anyhow::Result<PathBuf> {
    rep.log("checking small stock images");
    download::ensure_stock(rep, settings)
}

fn backup_opt_in(
    rep: &StdoutReporter,
    settings: &Settings,
    state: &State,
) -> anyhow::Result<Option<PathBuf>> {
    match state {
        State::Recovery(_) | State::Android(_) => {}
        _ => {
            rep.log("Stock backup skipped: backup needs stock Android/recovery adb.");
            return Ok(None);
        }
    }
    if !ask_yes(
        "Back up the stock partitions first? Takes 10 to 20 minutes and about 16 GB. This keeps device-specific calibration and serial data. The public stock dump is only model-level.",
        false,
    ) {
        return Ok(None);
    }

    match state {
        State::Recovery(_) => {
            rep.log("Rebooting to stock Android for the DSU backup.");
            let _ = core::proc::adb_ok(rep, &["reboot"]);
            rep.log("If Android shows a data error, use Recovery -> Factory data reset first.");
            device::wait_for_adb(rep, 1200, true)?;
        }
        State::Android(_) => {}
        _ => unreachable!(),
    }
    rep.log("The tablet will boot a temporary userdebug GSI for the dump.");
    let dir = core::backup::backup(rep, settings, None)?;
    rep.log(&format!("stock backup saved to {}", dir.display()));
    Ok(Some(dir))
}

fn install(
    rep: &StdoutReporter,
    settings: &Settings,
    state: &State,
    variant: Variant,
    root: bool,
    backup_dir: Option<&Path>,
) -> anyhow::Result<()> {
    let (slot, start_summary) = match state {
        State::Recovery(s) | State::Android(s) => (slot_name(s), Some(s.clone())),
        State::Fastbootd => {
            let slot = device::fastboot_getvar(rep, "current-slot")?;
            let slot = slot.trim_start_matches('_').to_string();
            if slot != "a" && slot != "b" {
                anyhow::bail!("cannot determine active slot from fastbootd");
            }
            (slot, None)
        }
        _ => anyhow::bail!("install needs recovery adb, Android adb, or fastbootd"),
    };
    if slot != "a" && slot != "b" {
        anyhow::bail!("cannot determine active slot");
    }

    if let Some(s) = &start_summary {
        rep.log(&format!(
            "tablet: {} / {} slot {} lock {}",
            s.model, s.board, s.slot, s.locked
        ));
    }

    let stock = ensure_stock(rep, settings)?;
    let vbmeta = stock.join(format!("vbmeta_{slot}.img"));
    let boot = stock.join(format!("boot_{slot}.img"));
    let init_boot = stock.join(format!("init_boot_{slot}.img"));

    rep.log("");
    rep.log("Install plan");
    rep.log(&format!("  variant: {}", variant.label()));
    rep.log(&format!("  active slot: {slot}"));
    rep.log("  kernel: recommended Google GKI");
    rep.log("  vendor modules: stock");
    rep.log("  other slot: stock, left untouched");
    rep.log(&format!(
        "  stock backup: {}",
        if backup_dir.is_some() {
            "made"
        } else {
            "skipped"
        }
    ));
    rep.log(&format!("  root: {}", if root { "Magisk" } else { "no" }));
    rep.log("");
    if ask("Type INSTALL to continue", "") != "INSTALL" {
        anyhow::bail!("install not confirmed");
    }

    flash::install_fastbootd(rep, settings, &slot, variant, &vbmeta, &boot, true, false)?;

    rep.log("");
    rep.log("On the tablet: Recovery -> Factory data reset -> confirm.");
    rep.log("Waiting for the GSI to boot...");
    device::wait_for_adb(rep, 3600, true)?;

    if root {
        let stock_init = if let Some(dir) = backup_dir {
            dir.join(format!("init_boot_{slot}.img"))
        } else {
            init_boot.clone()
        };
        if !stock_init.exists() {
            anyhow::bail!("root needs stock init_boot_{slot}.img; none found");
        }
        rep.log("Installing Magisk root on the new GSI.");
        magisk::install_from_init_boot(rep, settings, &stock_init, &slot, None, false)?;
    }

    let now = device::require_model(rep)?;
    rep.log("");
    rep.log("Install complete");
    rep.log(&format!("  slot: {}", now.slot));
    rep.log(&format!(
        "  lineage: {}",
        if now.lineage.is_empty() {
            "GSI"
        } else {
            &now.lineage
        }
    ));
    rep.log(&format!("  lock state: {}", now.locked));
    if let Ok(uname) = core::proc::adb(rep, &["shell", "uname", "-r"]) {
        rep.log(&format!("  kernel: {}", uname.trim()));
    }
    rep.log("Check touch, rotation, speakers, cameras, Bluetooth, charging and sleep.");
    Ok(())
}

pub fn run(settings: &Settings) -> anyhow::Result<()> {
    let rep = StdoutReporter;
    rep.log("onn11core wizard");
    rep.log("Device: onn 11 Core (T405A). This unlocks and installs LineageOS.");
    rep.log("Never flash preloader, lk, GPT, seccfg, TEE or GZ.");
    rep.log("");

    let variant = match ask("Variant: 1 microG, 2 vanilla, 3 GApps", "1").as_str() {
        "1" | "microg" | "microG" => Variant::MicroG,
        "2" | "vanilla" => Variant::Vanilla,
        "3" | "gapps" => Variant::Gapps,
        other => anyhow::bail!("unknown variant: {other}"),
    };
    let root = ask_yes("Install Magisk root on the final GSI?", false);

    let mut state = wait_state(&rep, true)?;
    match &state {
        State::Recovery(s) | State::Android(s) => {
            if s.locked != "0" {
                unlock_from_adb(&rep, s)?;
                state = wait_state(&rep, false)?;
            }
        }
        State::Lk => {
            unlock_from_lk(&rep)?;
            state = wait_state(&rep, false)?;
        }
        State::Fastbootd => {
            let unlocked = device::fastboot_getvar(&rep, "unlocked")?;
            if !unlocked.to_lowercase().contains("yes") {
                anyhow::bail!("fastbootd is running but the bootloader is locked");
            }
        }
        _ => {}
    }

    if matches!(state, State::Lk) {
        unlock_from_lk(&rep)?;
        state = wait_state(&rep, false)?;
    }

    let backup_dir = backup_opt_in(&rep, settings, &state)?;
    match &state {
        State::Recovery(_) | State::Android(_) | State::Fastbootd => {}
        _ => {
            state = wait_state(&rep, false)?;
        }
    }
    install(&rep, settings, &state, variant, root, backup_dir.as_deref())
}

pub fn probe(settings: &Settings) -> anyhow::Result<()> {
    let _ = settings;
    let rep = StdoutReporter;
    match detect(&rep)? {
        State::Recovery(s) | State::Android(s) => {
            rep.log("state: adb");
            rep.log(&format!("model: {} / {}", s.model, s.board));
            rep.log(&format!("fingerprint: {}", s.fingerprint));
            rep.log(&format!("slot: {}", s.slot));
            rep.log(&format!("lock: {}", s.locked));
            rep.log(&format!("verified boot: {}", s.verified_boot));
            if !s.lineage.is_empty() {
                rep.log(&format!("lineage: {}", s.lineage));
            }
        }
        State::Fastbootd => {
            rep.log("state: fastbootd");
            rep.log(&format!(
                "current-slot: {}",
                device::fastboot_getvar(&rep, "current-slot")?
            ));
            rep.log(&format!(
                "unlocked: {}",
                device::fastboot_getvar(&rep, "unlocked")?
            ));
            rep.log(&format!(
                "product: {}",
                device::fastboot_getvar(&rep, "product")?
            ));
        }
        State::Lk => {
            rep.log("state: LK fastboot");
            let info = unlock::read_lk_info(&rep, 10)?;
            rep.log(&format!("version: {}", info.version));
            rep.log(&format!("product: {}", info.product));
            rep.log(&format!("slot: {}", info.slot));
            rep.log(&format!("unlocked: {}", info.unlocked));
        }
        State::Sideload => rep.log("state: adb sideload"),
        State::Unauthorized => rep.log("state: adb unauthorized"),
        State::Unknown => rep.log("state: adb connected, boot state unknown"),
        State::None => {
            rep.log("state: no device");
            rep.log("Boot stock recovery with Volume Up + USB, or Android with USB debugging.");
        }
    }
    Ok(())
}
