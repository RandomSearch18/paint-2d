# Contributing to Paint 2D

## Cross-compilation

We use [`cross`](https://github.com/cross-rs/cross) to build the app for different platforms. Follow their instructions to install it, and then build the app for the desired platform, e.g.

```bash
cross build --release --target aarch64-unknown-linux-gnu
```

Building for macOS targets requires some [convoluted steps](https://github.com/cross-rs/cross-toolchains#apple-targets) to be followed if you're not using a macOS device (blame Apple licences).

## Tested terminals

Here's a list of terminals that Paint 2D has been tested in. Please test the app on your own platforms and favourite terminals so that the results can be added to the list, and any compatibility issues fixed!

### Linux

- ✅ Xfce Terminal Emulator
  - Subjectively looks the nicest in this one
- ✅ Konsole
- ✅ Visual Studio Code integrated terminal
- ✅ Kitty

### Windows

- ✅ Default Windows console
- ❌ WINE console
  - Doesn't seem to support ANSI escape codes

### macOS

- ✅ macOS Terminal

## Publishing a release

1. Bump version in `Cargo.toml`
2. Make sure `Cargo.lock` has updated with the new version number (done automatically if you use rust-analyzer)
3. Commit the version bump to the `master` branch
4. Wait for the GitHub Action to build all the binaries for the various platforms
5. Download all the zip file artifacts from the GitHub Action
6. Create a GitHub release, tell it to create a new tag (e.g. `v0.4.1`), upload the zip files to the assets/binaries section, write a little changelog, and publish it as the latest release
7. ~~Profit?~~ (Disclaimer: There is no profit, just labour of love)
