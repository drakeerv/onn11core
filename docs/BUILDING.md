# Building

## Tool

Requirements: Rust and platform-tools on PATH.

Linux needs libusb headers:

```sh
sudo apt install libusb-1.0-0-dev
cargo build --release
```

The binary is `target/release/onn11core`.

The release has two Linux builds, both with the bundled libusb:

```sh
# glibc
cargo build --release --target x86_64-unknown-linux-gnu --features vendored-libusb

# musl, static
sudo apt install musl-tools
rustup target add x86_64-unknown-linux-musl
CC_x86_64_unknown_linux_musl=musl-gcc AR_x86_64_unknown_linux_musl=ar CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=musl-gcc PKG_CONFIG_LIBDIR=/tmp/empty-pkgconfig PKG_CONFIG_PATH=/tmp/empty-pkgconfig cargo build --release --target x86_64-unknown-linux-musl --features vendored-libusb
```

The empty pkg-config path disables libudev in the musl build. The musl binary
is fully static and has no runtime libc or libusb dependency.

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
device branch. drakeerv obtained the Tinno source code from Walmart and
published it there. The release boot image was built from the AOSP GKI
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
