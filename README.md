# onn11core

A guided installer for the onn 11" Core (2026) tablet. One command walks through
the unlock, optional stock backup, LineageOS GSI, recommended Google GKI kernel,
and optional Magisk root.

This is a stopgap until the official LineageOS port. It is not a full ROM.

## Warning

This is device specific and destructive. It only works on the onn 11" Core,
model 36018341, device onn11Core, board T405A.

Never flash preloader, lk, GPT, seccfg, TEE or GZ. A bad write there has no
recovery path. The unlock is irreversible in practice and wipes userdata.

## Requirements

- Linux or Windows. Linux is the tested path.
- Android platform-tools (adb and fastboot) on PATH.
- Stock recovery adb. The recommended entry is to power off, hold Volume Up, and
  plug in USB. No developer settings are needed.
- Windows needs a libusb/WinUSB driver for the raw USB unlock. WSL2 with usbipd
  is the easiest route.
- The wizard detects whether the bootloader is locked. No flag is required.

## Quick start

Download the latest release from
https://github.com/drakeerv/onn11core/releases/latest and run:

```sh
onn11core
```

Linux has two builds. Use `onn11core-linux-x86_64-musl.tar.gz` for a static
binary, or `onn11core-linux-x86_64-glibc.tar.gz` for a normal glibc system.

The wizard asks:

1. LineageOS variant: microG, vanilla, or GApps.
2. Install Magisk root? Default no.
3. Back up the stock partitions? Default no. The backup takes 10 to 20 minutes
   and keeps per-device calibration and serial data.

It then detects recovery adb, Android adb, fastbootd, or raw LK fastboot,
unlocks if needed, writes the physical images in LK fastboot, and writes the
system image in fastbootd. The first LK flash starts immediately after the
reboot and waits for the tablet. No unplug is needed unless the command was
started late.

The GKI kernel is flashed by default. It includes the GhostLock fix and other
security backports. The stock kernel is only an advanced fallback. Stock vendor
modules stay in place.

After the flash, use Recovery and choose Factory data reset. This step is
required on this tablet. The stock recovery may show a "Try again" / "Factory
data reset" screen. Choose Factory data reset with the volume keys, press
Power, then confirm. Do not choose Try again. This initializes metadata
encryption.

## What the wizard does

- If adb is already available, the wizard uses it.
- On a locked tablet with no adb, the wizard waits for LK fastboot. Turn the
  tablet off, then plug it in while holding Volume Up + Volume Down. It reads
  and validates LK before unlocking.
- After the unlock confirmation, unplug the tablet. The unlocker waits, then
  plug it back in for the raw payload.
- The raw USB unlock and the physical image writes use LK. The system image and
  the final wipe use fastbootd. The wizard starts the first LK flash immediately
  and waits for the tablet.
- The active slot gets the GSI, GKI, and vbmeta. The other slot stays stock.
- LK fastboot is used for vbmeta, boot, init_boot, and dtbo. fastbootd is used
  for the system image and the final wipe.
- The small stock bundle supplies model-level stock vbmeta, boot, and init_boot
  images for the fast path and root.
- The stock backup is optional and per-device.

## Vendor modules

Vendor modules stay stock. The stopgap does not package the source-built vendor
modules. They are build artifacts for the official LineageOS port.

## Probe

A read-only state probe is available:

```sh
onn11core probe
```

It prints the detected state, model, slot, lock state, and LK or fastbootd
details without changing anything.

## Manual

The plain fastboot path is in [docs/MANUAL.md](docs/MANUAL.md).
Build instructions are in [docs/BUILDING.md](docs/BUILDING.md).

## Stock dump

The stock dump is my own dump, hosted at
[el-vertedero/onn_onn11core_dump](https://github.com/el-vertedero/onn_onn11core_dump).

It is a reference tree, not a flashable bundle. It includes boot chain files
that must not be flashed. It is model-level and does not replace a per-device
backup.

## Build

The tool is Rust. See [docs/BUILDING.md](docs/BUILDING.md).

```sh
cargo build --release
```

The released GKI boot image is built from Google AOSP `common-android16-6.12`,
pinned to `android16-6.12-2025-09`, plus the compatibility patch. It does not use
the Tinno source dump.

## Credits

- **drakeerv**: [GitHub](https://github.com/drakeerv), [Nura Wiki](https://wiki.nura.eco/wiki/Onn_11_Core_Tablet_%28onn-t405a%29) - unlock exploit, device work, stock dump, Tinno source code from Walmart.
- **SelfhostedPro**: [GitHub](https://github.com/SelfhostedPro), [t405a-gsi](https://github.com/SelfhostedPro/t405a-gsi) - MIT licensed core flashing logic.
- **el-vertedero**: [onn_onn11core_dump](https://github.com/el-vertedero/onn_onn11core_dump) - hosting my stock dump.
- **TrebleDroid**: [treble_experimentations](https://github.com/TrebleDroid/treble_experimentations) - DSU userdebug GSI used by the stock backup.
- **MisterZtr**: [LineageOS_gsi](https://github.com/MisterZtr/LineageOS_gsi) - vanilla LineageOS GSI.
- **LineageOS for microG**: [SourceForge](https://sourceforge.net/projects/lineageos-4-microg/) - microG GSI.
- **Google AOSP**: [kernel/common](https://android.googlesource.com/kernel/common/) - GKI kernel source for the released boot image.
- **onn-oss**: [kernel-6.12](https://github.com/onn-oss/kernel-6.12) - device kernel and module source archive for the module build and official port.

License: MIT. See LICENSE.
