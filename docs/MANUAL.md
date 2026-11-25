# Manual method

This is the plain fastboot version of the install. The tool automates the same
steps and is the recommended path for the unlock and the stock backup.

## 1. Unlock

The unlock uses a raw USB payload. Plain `fastboot` cannot send it. Use the
tool:

```sh
onn11core unlock
```

The tablet will reboot to recovery. Choose Factory data reset, then boot the
stock system and enable USB debugging again.

Verify:

```sh
adb shell getprop ro.boot.flash.locked
# 0

adb shell getprop ro.boot.verifiedbootstate
# orange
```

## 2. Backup

Use the tool. It boots a DSU userdebug GSI, gets root, and dumps the stock
partitions:

```sh
onn11core backup
```

Keep the backup directory. It contains `vbmeta_<slot>.img`, which the manual
flash needs. The stock dump linked from the README can be used as a reference.
It is my dump, hosted by el-vertedero. Do not flash its `lk`,
preloader, TEE or other boot chain files.

## 3. Choose a slot

```sh
adb shell getprop ro.boot.slot_suffix
```

This returns `_a` or `_b`. Use that letter as `<slot>` below.

## 4. Download a GSI

Use the release images:

- `LineageOS-23.2-20260524-VANILLA-EXT4-GSI.7z`
- `LineageOS-23.2-2026.11.18-microG-gsi-btfixed.img.xz` (Bluetooth fix included)

Extract the image if needed. The result must be a raw ext4 arm64 system image.

## 5. Patch vbmeta

The stock vbmeta image comes from your backup. LK fastboot on this tablet needs
the command started before the tablet reconnects:

1. Unplug the tablet.
2. Start the command below, then plug the tablet back in.
3. When fastboot says it is waiting for a device, the command will fire.

```sh
fastboot --disable-verity --disable-verification flash vbmeta_<slot> /path/to/backup/vbmeta_<slot>.img
```

## 6. Flash the system image

Boot to fastbootd:

```sh
adb reboot fastboot
fastboot devices
fastboot getvar is-userspace
# yes
```

Free space and resize:

```sh
fastboot delete-logical-partition product_<slot>
fastboot resize-logical-partition system_<slot> <image size plus 64 MiB, at least 3000000000>
```

Flash and wipe:

```sh
fastboot flash system_<slot> /path/to/system.img
fastboot -w
fastboot reboot recovery
```

In Recovery choose Factory data reset. This initializes metadata encryption.
Without it the next boot can fail with `init_user0_failed`. If the stock recovery
shows a "Try again" / "Factory data reset" screen, choose Factory data reset
with the volume keys, press Power, then confirm. Do not choose Try again.

## 7. GKI kernel (recommended)

The Google GKI kernel includes the GhostLock fix and other security backports
from the newer GKI branch. The stock kernel is only a fallback. The stock vendor
modules stay in place. To flash the GKI boot image from the release:

1. Unplug the tablet.
2. Start the command, then plug the tablet back in.

```sh
fastboot flash boot_<slot> /path/to/boot_b_gki.img
fastboot reboot
```

Revert with the stock boot image from your backup:

```sh
fastboot flash boot_<slot> /path/to/backup/boot_<slot>.img
```

## Restore stock

Restore the physical partitions in LK fastboot, then the super image:

```sh
fastboot flash boot_<slot> /path/to/backup/boot_<slot>.img
fastboot flash init_boot_<slot> /path/to/backup/init_boot_<slot>.img
fastboot flash vendor_boot_<slot> /path/to/backup/vendor_boot_<slot>.img
fastboot flash dtbo_<slot> /path/to/backup/dtbo_<slot>.img
fastboot flash vbmeta_<slot> /path/to/backup/vbmeta_<slot>.img
fastboot flash vbmeta_system_<slot> /path/to/backup/vbmeta_system_<slot>.img
fastboot flash vbmeta_vendor_<slot> /path/to/backup/vbmeta_vendor_<slot>.img
fastboot flash super /path/to/backup/super.img
fastboot reboot recovery
```

If the full super flash fails, use the logical restore:

```sh
lpunpack --slot=<0 for a, 1 for b> \
  -p system_<slot> -p product_<slot> -p vendor_<slot> \
  -p system_ext_<slot> -p vendor_dlkm_<slot> \
  -p system_dlkm_<slot> -p odm_dlkm_<slot> \
  /path/to/backup/super.img /tmp/stock-parts

fastboot reboot fastboot
fastboot delete-logical-partition system_<slot>
fastboot create-logical-partition system_<slot> <size>
fastboot flash system_<slot> /tmp/stock-parts/system_<slot>.img
```

Repeat the delete, create and flash steps for each partition. This is the same
path as `onn11core restore --logical` on Linux.

Never flash preloader, lk, GPT, seccfg, TEE or GZ.
