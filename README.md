# onn11core

A small command line installer for the onn 11" Core (2026) tablet. It unlocks
the bootloader, backs up the stock partitions, and installs LineageOS. It also
handles the optional GKI kernel and a stock restore.

This is a stopgap until the official LineageOS port. It is not a full ROM.

## Warning

This is device specific and destructive. It only works on the onn 11" Core,
model 36018341, device onn11Core, board T405A.

Never flash preloader, lk, GPT, seccfg, TEE or GZ. A bad write there has no
recovery path. The unlock is irreversible in practice and wipes userdata.

## Requirements

- Linux or Windows. Linux is the tested path.
- Android platform-tools (adb and fastboot).
- USB debugging enabled, or the tablet in fastboot mode.
- Windows needs a libusb/WinUSB driver for the raw USB unlock. WSL2 with usbipd
  is the easiest route.
- An unlocked bootloader for backup and flash. If it is locked, unlock first.

## Quick start

Download the latest release and run:

```sh
onn11core check
onn11core unlock
onn11core backup
onn11core flash --backup /path/to/backup --variant microg
```

The one-shot path is:

```sh
onn11core all --variant microg
```

Use `--variant vanilla` for LineageOS without microG. The GApps GSI is also
available with `--variant gapps`.

After the flash, use Recovery and choose Factory data reset. This step is
required on this tablet.

If you want the GKI kernel after the GSI boots:

```sh
onn11core gki
```

This is optional. The stock kernel remains the safe default.

## Manual method

The same steps are documented as plain fastboot commands in
[docs/MANUAL.md](docs/MANUAL.md). The tool is still the safest way to run the
raw USB unlock and the stock backup.

## Stock dump

The public stock dump is at
https://github.com/el-vertedero/onn_onn11core_dump

It is a reference tree, not a flashable bundle. It includes boot chain files
that must not be flashed. The tool takes its own stock backup with a DSU before
flashing. Keep that backup, it is your restore path.

## Build

The tool is Rust. See [docs/BUILDING.md](docs/BUILDING.md).

```sh
cargo build --release
```

The optional GKI kernel is built from the public onn-oss kernel source. The
release includes a prebuilt boot image.

## Credits

Unlock exploit and device work: drakeerv.

Core flashing logic based on the MIT licensed t405a-gsi project by
SelfhostedPro.

Stock dump reference: el-vertedero.

GSIs: MisterZtr LineageOS GSI and LineageOS for microG.

Kernel source: onn-oss kernel-6.12.

License: MIT. See LICENSE.
