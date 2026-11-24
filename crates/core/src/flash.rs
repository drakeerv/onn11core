//! Flash a GSI, the optional GKI kernel, or restore stock from a backup.

use crate::backup::{self, BackupManifest};
use crate::consts::{self, Variant};
use crate::download;
use crate::proc;
use crate::report::Reporter;
use crate::{device, Result, Settings};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn parse_size(value: &str) -> Option<u64> {
    let v = value.trim();
    let v = v.strip_prefix("0x").unwrap_or(v);
    u64::from_str_radix(v, 16).ok()
}

fn backup_slot(backup_dir: &Path, manifest: &BackupManifest) -> Result<String> {
    backup::backup_slot(backup_dir, manifest)
}

fn system_name(slot: &str) -> String {
    format!("system_{slot}")
}

/// Flash a raw ext4 GSI. The vbmeta patch runs in LK fastboot with the replug
/// prompt; the system write runs in fastbootd.
pub fn flash(
    rep: &dyn Reporter,
    settings: &Settings,
    backup_dir: &Path,
    image: Option<&Path>,
    variant: Variant,
    force: bool,
    dry_run: bool,
) -> Result<()> {
    let manifest = backup::load_backup(backup_dir)?;
    backup::verify_backup(rep, backup_dir, &manifest)?;
    let slot = backup_slot(backup_dir, &manifest)?;
    let system = system_name(&slot);
    let product = format!("product_{slot}");
    let vbmeta_name = format!("vbmeta_{slot}");
    let vbmeta = backup_dir.join(format!("{vbmeta_name}.img"));
    if !vbmeta.exists() {
        anyhow::bail!("backup is missing {vbmeta_name}.img");
    }

    let (image, label) = match image {
        Some(path) => (
            download::ensure_custom_image(rep, path)?,
            "Custom GSI".to_string(),
        ),
        None => (
            download::ensure_lineage(rep, settings, variant)?,
            variant.label().to_string(),
        ),
    };
    let image_size = fs::metadata(&image)?.len();
    rep.log(&format!(
        "{label}: {} ({image_size} bytes)",
        image.display()
    ));

    if dry_run {
        rep.log("dry run; would run:");
        rep.log("  adb reboot fastboot");
        rep.log(&format!("  fastboot delete-logical-partition {product}"));
        rep.log(&format!(
            "  fastboot resize-logical-partition {system} <size>   # if needed"
        ));
        rep.log(&format!("  fastboot flash {system} {}", image.display()));
        rep.log("  fastboot reboot bootloader");
        rep.log("  unplug -> Enter -> plug, then:");
        rep.log(&format!(
            "  fastboot --disable-verity --disable-verification flash {vbmeta_name} {}",
            vbmeta.display()
        ));
        rep.log("  fastboot reboot recovery, then Factory data reset");
        return Ok(());
    }

    let summary = device::require_model(rep)?;
    if summary.locked != "0" {
        anyhow::bail!("bootloader is locked; unlock first");
    }
    let device_slot = summary.slot.trim_start_matches('_');
    if !device_slot.is_empty() && device_slot != slot {
        anyhow::bail!(
            "backup is slot {slot} but the device is slot {device_slot}; make a matching backup"
        );
    }
    if !summary.lineage.is_empty() && !force {
        anyhow::bail!("a LineageOS/GSI system is already running; pass --force to replace it");
    }

    device::ensure_adb(rep)?;
    proc::adb_ok(rep, &["reboot", "fastboot"])?;
    device::wait_for_fastboot(rep, 180)?;
    if device::fastboot_getvar(rep, "is-userspace")?.to_lowercase() != "yes" {
        anyhow::bail!("not in fastbootd (is-userspace != yes); run `adb reboot fastboot` again");
    }

    let product_size = parse_size(&device::fastboot_getvar(
        rep,
        &format!("partition-size:{product}"),
    )?)
    .unwrap_or(0);
    if product_size > 0 {
        rep.log(&format!("deleting {product} to free super space"));
        let _ = proc::fastboot_ok(rep, &["delete-logical-partition", &product]);
    }

    let current = parse_size(&device::fastboot_getvar(
        rep,
        &format!("partition-size:{system}"),
    )?)
    .unwrap_or(0);
    rep.log(&format!("{system} size: {current} bytes"));
    if current < image_size {
        let target = std::cmp::max(image_size + 64 * 1024 * 1024, 3_000_000_000);
        rep.log(&format!("resizing {system} to {target} bytes"));
        proc::fastboot_ok(
            rep,
            &["resize-logical-partition", &system, &target.to_string()],
        )?;
        let now = parse_size(&device::fastboot_getvar(
            rep,
            &format!("partition-size:{system}"),
        )?)
        .unwrap_or(0);
        if now < image_size {
            anyhow::bail!("{system} is still too small ({now} < {image_size})");
        }
    }

    let image_path = image
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("non-UTF8 image path"))?;
    rep.log(&format!("flashing {system}"));
    proc::fastboot_ok(rep, &["flash", &system, image_path])?;

    rep.log("rebooting to bootloader to patch vbmeta");
    proc::fastboot_ok(rep, &["reboot", "bootloader"])?;
    device::wait_for_fastboot(rep, 180)?;
    device::replug_prompt(rep);
    rep.log("starting vbmeta flash; plug the tablet in when it is waiting");
    let vbmeta_path = vbmeta
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("non-UTF8 vbmeta path"))?;
    proc::fastboot_ok(
        rep,
        &[
            "--disable-verity",
            "--disable-verification",
            "flash",
            &vbmeta_name,
            vbmeta_path,
        ],
    )?;

    rep.log("");
    rep.log("On the tablet: Recovery -> Wipe data/factory reset -> Factory data reset.");
    proc::fastboot_ok(rep, &["reboot", "recovery"])?;
    device::wait_for_adb(rep, 3600, true)?;
    let now = device::summary(rep)?;
    if !now.lineage.is_empty() {
        rep.log(&format!("LineageOS booted: {}", now.lineage));
    } else {
        rep.log(&format!(
            "GSI booted: fingerprint={:?} slot={}",
            now.fingerprint, now.slot
        ));
    }
    rep.log("Check touch, rotation, speakers, cameras, Bluetooth, charging and sleep.");
    Ok(())
}

/// Flash the Google GKI boot image to the active slot. Stock vendor modules
/// and the stock vendor_boot remain in place.
pub fn flash_gki(
    rep: &dyn Reporter,
    settings: &Settings,
    image: Option<&Path>,
    dry_run: bool,
) -> Result<()> {
    if dry_run {
        let slot = "b";
        rep.log(&format!(
            "dry run; would flash {} to boot_{slot} and reboot",
            image
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "the released GKI image".to_string())
        ));
        return Ok(());
    }

    let summary = device::require_model(rep)?;
    let slot = summary.slot.trim_start_matches('_').to_string();
    if slot != "a" && slot != "b" {
        anyhow::bail!("cannot determine the active slot");
    }
    if summary.locked != "0" {
        anyhow::bail!("bootloader is locked; unlock first");
    }

    let image = match image {
        Some(path) => path
            .canonicalize()
            .map_err(|e| anyhow::anyhow!("cannot open {}: {e}", path.display()))?,
        None => download::ensure_gki(rep, settings)?,
    };
    let image_str = image
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("non-UTF8 image path"))?;

    device::ensure_adb(rep)?;
    proc::adb_ok(rep, &["reboot", "bootloader"])?;
    device::wait_for_fastboot(rep, 180)?;
    device::replug_prompt(rep);
    rep.log("starting GKI boot flash; plug the tablet in when it is waiting");
    proc::fastboot_ok(rep, &["flash", &format!("boot_{slot}"), image_str])?;
    proc::fastboot_ok(rep, &["reboot"])?;
    rep.log("GKI kernel flashed. Revert with the stock boot image from your backup.");
    Ok(())
}

fn run_lpunpack(super_img: &Path, outdir: &Path, slot: &str) -> Result<Vec<(String, PathBuf)>> {
    fs::create_dir_all(outdir)?;
    let parts = [
        "system",
        "product",
        "vendor",
        "system_ext",
        "vendor_dlkm",
        "system_dlkm",
        "odm_dlkm",
    ];
    let mut args = vec![format!("--slot={}", consts::slot_number(slot))];
    for part in parts {
        args.push("-p".to_string());
        args.push(format!("{part}_{slot}"));
    }
    args.push(super_img.display().to_string());
    args.push(outdir.display().to_string());
    let status = Command::new("lpunpack")
        .args(&args)
        .status()
        .map_err(|e| anyhow::anyhow!("cannot run lpunpack: {e}"))?;
    if !status.success() {
        anyhow::bail!("lpunpack failed; install it or use the full super restore");
    }
    let mut out = Vec::new();
    for part in parts {
        let name = format!("{part}_{slot}");
        let path = outdir.join(format!("{name}.img"));
        if !path.exists() {
            anyhow::bail!("lpunpack did not produce {name}.img");
        }
        out.push((name, path));
    }
    Ok(out)
}

fn restore_logical(
    rep: &dyn Reporter,
    settings: &Settings,
    backup_dir: &Path,
    slot: &str,
) -> Result<()> {
    let super_img = backup_dir.join("super.img");
    if !super_img.exists() {
        anyhow::bail!("backup has no super.img; cannot use --logical");
    }
    let outdir = settings.workdir.join("restore-logical");
    let parts = run_lpunpack(&super_img, &outdir, slot)?;

    proc::fastboot_ok(rep, &["reboot", "fastboot"])?;
    device::wait_for_fastboot(rep, 180)?;
    if device::fastboot_getvar(rep, "is-userspace")?.to_lowercase() != "yes" {
        anyhow::bail!("could not enter fastbootd for the logical restore");
    }

    for (name, path) in parts {
        let size = fs::metadata(&path)?.len();
        rep.log(&format!("restoring {name} ({size} bytes)"));
        let _ = proc::fastboot_ok(rep, &["delete-logical-partition", &name]);
        proc::fastboot_ok(rep, &["create-logical-partition", &name, &size.to_string()])?;
        proc::fastboot_ok(rep, &["flash", &name, &path.display().to_string()])?;
    }
    Ok(())
}

/// Restore stock boot/vbmeta/super from a backup.
pub fn restore(
    rep: &dyn Reporter,
    settings: &Settings,
    backup_dir: &Path,
    no_super: bool,
    logical: bool,
    dry_run: bool,
) -> Result<()> {
    let manifest = backup::load_backup(backup_dir)?;
    backup::verify_backup(rep, backup_dir, &manifest)?;
    let slot = backup_slot(backup_dir, &manifest)?;
    let physicals = consts::restore_physical(&slot);

    if dry_run {
        rep.log("dry run; would run (LK fastboot):");
        for name in &physicals {
            rep.log(&format!(
                "  fastboot flash {name} {}/{name}.img",
                backup_dir.display()
            ));
        }
        if logical {
            rep.log("  fastboot reboot fastboot");
            rep.log("  extract logical partitions with lpunpack, then flash them in fastbootd");
        } else if !no_super {
            rep.log(&format!(
                "  fastboot flash super {}/super.img",
                backup_dir.display()
            ));
        }
        rep.log("  fastboot reboot recovery, then Factory data reset");
        return Ok(());
    }

    device::ensure_fastboot(rep)?;
    device::replug_prompt(rep);

    for (i, name) in physicals.iter().enumerate() {
        let path = backup_dir.join(format!("{name}.img"));
        if !path.exists() {
            anyhow::bail!("backup is missing {}", path.display());
        }
        if i == 0 {
            rep.log("starting the first stock flash; plug the tablet in when it is waiting");
        }
        proc::fastboot_ok(rep, &["flash", name, &path.display().to_string()])?;
    }

    if logical {
        restore_logical(rep, settings, backup_dir, &slot)?;
    } else if !no_super {
        let super_img = backup_dir.join("super.img");
        if !super_img.exists() {
            anyhow::bail!("backup is missing {}", super_img.display());
        }
        if let Err(err) =
            proc::fastboot_ok(rep, &["flash", "super", &super_img.display().to_string()])
        {
            anyhow::bail!(
                "super restore failed: {err}\nRerun with --logical (Linux, needs lpunpack) or use the manual recovery guide."
            );
        }
    }

    rep.log("");
    rep.log("On the tablet: Recovery -> Wipe data/factory reset -> Factory data reset.");
    proc::fastboot_ok(rep, &["reboot", "recovery"])?;
    Ok(())
}
