# Memory Pak

A cross-platform game collection tracker built with Rust, egui, and eframe. Memory Pak allows you to track which consoles and games you own, have favorited, or want on your wishlist.

[![Crates.io](https://img.shields.io/crates/v/memory-pak?style=for-the-badge)](https://crates.io/crates/memory-pak)
[![Crates.io Downloads](https://img.shields.io/crates/d/memory-pak?style=for-the-badge)](https://crates.io/crates/memory-pak)
[![GitHub Release](https://img.shields.io/github/v/release/Aspenini/Memory-Pak?style=for-the-badge)](https://github.com/Aspenini/Memory-Pak/releases)
[![GitHub Downloads](https://img.shields.io/github/downloads/Aspenini/Memory-Pak/total?style=for-the-badge)](https://github.com/Aspenini/Memory-Pak/releases)
[![CI](https://github.com/Aspenini/Memory-Pak/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/Aspenini/Memory-Pak/actions/workflows/ci.yml)
[![Deploy Website](https://github.com/Aspenini/Memory-Pak/actions/workflows/deploy.yml/badge.svg)](https://github.com/Aspenini/Memory-Pak/actions/workflows/deploy.yml)

## Features

- **Two Main Tabs**: Consoles and Games, both with pagination
- **Collector Tabs**: LEGO Dimensions and Skylanders tracking
- **Console States**: Mark consoles as Owned, Favorite, or Wishlist, with notes
- **Game States**: Mark games as Owned, Favorite, or Wishlist, with notes
- **Embedded Game Database**: Game data is compiled directly into the binary
- **Stable ID System**: Games use content-based IDs so database updates do not break saved states
- **Cross-Console Search**: View all games across all consoles, or filter by one console
- **Sorting and Filtering**: Sort by title, release year, or status and filter by collection state
- **State Persistence**: Saves user data to platform-specific directories or browser localStorage
- **Import/Export**: Export your collection state to JSON and import it back

## Project Structure

```text
Memory-Pak/
├── Cargo.toml          # Project configuration
├── justfile            # Common build/check/package commands
├── src/
│   ├── lib.rs          # Shared app code and platform entrypoints
│   ├── main.rs         # Thin desktop/web binary wrapper
│   ├── console_data.rs # Console definitions
│   ├── game_data.rs    # Embedded game data loading
│   ├── persistence.rs  # Save/load user state
│   └── ui.rs           # UI rendering functions
└── database/           # JSON data embedded at compile time
    ├── nes.json
    ├── snes.json
    ├── n64.json
    └── ... (one JSON file per console)
```

## Building

### Just Recipes

Install [`just`](https://github.com/casey/just), then run:

```bash
just                 # list available recipes
just check           # cargo check
just build           # desktop release build
just web-build       # production WASM build
just android-build   # native Android APK via cargo-apk
```

### Desktop

```bash
cargo build --release
```

Binary output: `target/release/memory_pak[.exe]`

### Web Build (WASM)

```bash
# One-time setup
just install-web-tools

# Development with hot reload, opens at http://127.0.0.1:8080
just web-serve

# Production build, output in dist/
just web-build
```

### Android Native APK

Native Android compilation uses eframe's `android-native-activity` backend and [`cargo-apk`](https://github.com/rust-mobile/cargo-apk).

```bash
# One-time Rust-side setup
just install-android-tools

# One-time local release signing key setup
just android-keystore

# Requires Android SDK/NDK variables discoverable by cargo-apk.
# You will be prompted for the keystore password.
just android-build

# Build, install, and run on an attached device/emulator
just android-run
```

Android APKs must be signed to install or update, but this does not require a Google Play developer account. `just android-keystore` creates a local self-signed release keystore under `.android/`, which is ignored by Git. Keep that keystore and password safe; future direct APK updates for the same package name must be signed with the same key.

Configured Android targets:

- `aarch64-linux-android`
- `armv7-linux-androideabi`
- `x86_64-linux-android`

### Platform Installers

**Windows (MSI):**

```bash
cargo install cargo-wix
cargo wix
```

Output: `target/wix/Memory-Pak-0.1.5-x86_64.msi`
Requires: [WiX Toolset v3.11+](https://wixtoolset.org/)

**macOS (.app):**

```bash
cargo install cargo-bundle
cargo bundle --release
```

Output: `target/release/bundle/osx/Memory Pak.app`

**Linux (.deb):**

```bash
cargo install cargo-deb
cargo deb
```

Output: `target/debian/memory_pak_*.deb`

## User Data Storage

- **Desktop/Mobile**: Stored in platform-specific user data directories.
- **Web**: Stored in browser localStorage with keys like `memory_pak_state_{console_id}`.

## Export Format

```json
{
  "version": "1.0",
  "export_date": "2024-01-01T00:00:00Z",
  "console_states": [
    {
      "console_id": "nes",
      "owned": true,
      "favorite": false,
      "wishlist": false,
      "notes": "My original NES from 1985"
    }
  ],
  "consoles": [
    {
      "console_id": "nes",
      "games": [
        {
          "game_id": "nes-a1b2c3d4e5f6...",
          "owned": true,
          "favorite": false,
          "wishlist": false,
          "notes": ""
        }
      ]
    }
  ]
}
```

Game IDs use the stable ID format `{console_id}-{hash}` for compatibility across database updates.
