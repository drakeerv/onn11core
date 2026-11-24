# Building

## Tool

Requirements: Rust and platform-tools on PATH.

Linux needs libusb headers:

```sh
sudo apt install libusb-1.0-0-dev
cargo build --release
```

The binary is `target/release/onn11core`.

Linux release builds use the optional bundled libusb feature:

```sh
cargo build --release --features vendored-libusb
```

Windows uses the vendored libusb automatically. Build on Windows with the
normal Rust toolchain.

Run the tests:

```sh
cargo test
```

## Optional GKI kernel

The GKI kernel is not needed for the main install. The stock kernel and stock
vendor modules remain the safe default.

The public kernel source is at https://github.com/onn-oss/kernel-6.12 on the
device branch. The release boot image was built from the AOSP GKI
`android16-6.12` source plus the device module tree, using the build notes and
patches from the onn-clean kernel pipeline.

At a high level:

1. Sync the AOSP GKI manifest for `common-android16-6.12`.
2. Use the pinned KMI generation 5 kernel source and apply the compatibility
   patch for unsigned stock modules and the vendor task_struct size.
3. Build `//common:kernel_aarch64` with Kleaf and stamp the build.
4. Package the raw Image as a header v4 boot image with an empty ramdisk.

The device kernel source and module sources are on onn-oss. The stopgap does
not install rebuilt vendor modules. It keeps the stock vendor partitions.

## GSI

The GSI images are prebuilt. Building LineageOS is out of scope until the
official port.

## Release

Push a `v*` tag. The GitHub workflow builds Linux and Windows binaries and
attaches them to the release. The kernel and GSI images are attached to the
same release separately.
