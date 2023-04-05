# Connlib

Firezone's connectivity library shared by all clients.

## 🚧 Disclaimer 🚧

**NOTE**: This repository is undergoing heavy construction. You could say we're
_Building In The Open™_ in true open source spirit. Do not attempt to use
anything released here until this notice is removed. You have been warned.

## Building Connlib

1. You'll need a Rust toolchain installed if you don't have one already. We
   recommend following the instrucitons at https://rustup.rs.
1. Install relevant targets:

```
rustup target add \
  aarch64-apple-darwin \
  aarch64-apple-ios \
  aarch64-apple-ios-sim \
  aarch64-linux-android \
  aarch64-pc-windows-msvc \
  aarch64-unknown-linux-gnu \
  arm-linux-androideabi \
  armv7-linux-androideabi \
  i686-linux-android \
  i686-pc-windows-msvc \
  i686-unknown-linux-gnu \
  x86_64-apple-darwin \
  x86_64-apple-ios \
  x86_64-linux-android \
  x86_64-pc-windows-msvc \
  x86_64-unknown-linux-gnu
```

1. Follow the relevant instructions for your platform:
1. [Apple](#apple)
1. [Android](#android)
1. [Linux](#linux)
1. [Windows](#windows)

### Apple

Connlib should build successfully with recent macOS and Xcode versions assuming
you have Rust installed. If not, open a PR with the notes you found.

### Android

### Linux

### Windows
