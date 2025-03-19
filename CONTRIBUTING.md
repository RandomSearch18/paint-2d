# Contributing to Paint 2D

## Cross-compilation

We use [`cross`](https://github.com/cross-rs/cross) to build the app for different platforms. Follow their instructions to install it, and then build the app for the desired platform, e.g.

```bash
cross build --release --target aarch64-unknown-linux-gnu
```

Building for macOS targets requires some [additional steps](https://github.com/cross-rs/cross-toolchains#apple-targets) to be followed (blame Apple licences).
