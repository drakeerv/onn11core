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

Download the latest release from
https://github.com/drakeerv/onn11core/releases/latest and run:

Linux has two builds. Use `onn11core-linux-x86_64-musl.tar.gz` for a static
binary, or `onn11core-linux-x86_64-glibc.tar.gz` for a normal glibc system.

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

The stock dump is my own dump, hosted at
[el-vertedero/onn_onn11core_dump](https://github.com/el-vertedero/onn_onn11core_dump).

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

- **drakeerv**: [GitHub](https://github.com/drakeerv), [Nura Wiki](https://wiki.nura.eco/wiki/Onn_11_Core_Tablet_%28onn-t405a%29) - unlock exploit, device work, stock dump.
- **SelfhostedPro**: [GitHub](https://github.com/SelfhostedPro), [t405a-gsi](https://github.com/SelfhostedPro/t405a-gsi) - MIT licensed core flashing logic.
- **el-vertedero**: [onn_onn11core_dump](https://github.com/el-vertedero/onn_onn11core_dump) - hosting my stock dump.
- **TrebleDroid**: [treble_experimentations](https://github.com/TrebleDroid/treble_experimentations) - DSU userdebug GSI used by the stock backup.
- **MisterZtr**: [LineageOS_gsi](https://github.com/MisterZtr/LineageOS_gsi) - vanilla LineageOS GSI.
- **LineageOS for microG**: [SourceForge](https://sourceforge.net/projects/lineageos-4-microg/) - microG GSI.
- **onn-oss**: [kernel-6.12](https://github.com/onn-oss/kernel-6.12) - device kernel source for the GKI build.

License: MIT. See LICENSE.
