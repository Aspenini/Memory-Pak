# Memory Pak

[![CI](https://github.com/Aspenini/Memory-Pak/actions/workflows/ci.yml/badge.svg)](https://github.com/Aspenini/Memory-Pak/actions/workflows/ci.yml)
[![Deploy Website](https://github.com/Aspenini/Memory-Pak/actions/workflows/deploy.yml/badge.svg)](https://github.com/Aspenini/Memory-Pak/actions/workflows/deploy.yml)
[![License](https://img.shields.io/github/license/Aspenini/Memory-Pak)](LICENSE)

Memory Pak is a Rust and Slint game-collection tracker for desktop, web/PWA,
Android, and iOS. It tracks consoles, games, and toy-to-life collectibles using
independent owned, favorite, wishlist, and notes state.

## Architecture

```text
crates/
  memory_pak_catalog/  JSON validation and deterministic indexed catalog compiler
  memory_pak_core/     indexed queries, state merging, statistics, v2 backup I/O
  memory_pak_app/      Slint UI, lazy model, controllers, and platform services
database/              editable catalog source; never included in runtime packages
platform/android/      NativeActivity/Gradle shell and Storage Access Framework bridge
platform/ios/          Xcodegen project for the Rust Slint executable
web/                   minimal canvas bootstrap, manifest, and service worker
```

`memory_pak_core/build.rs` compiles the source JSON into a versioned
`catalog.bin`. The binary has a magic header, schema and source digests, a
deduplicated string table, dense records, n-gram search postings, facets, and
precomputed sort orders. Runtime artifacts embed only this immutable binary.

User data is a sparse v3 envelope. Entries contain only `owned`, `favorite`,
`wishlist`, and `notes`; default fields and empty entries are omitted. Unknown
stable IDs are retained during migration but excluded from visible statistics.
The deterministic v2 backup format remains the import/export format.

## Requirements

- Rust 1.94
- `wasm-pack` 0.14 for web builds
- `cargo-packager` for desktop packages
- Android SDK/NDK, Gradle, `cargo-ndk`, and Android Rust targets for Android
- Xcode and Xcodegen for iOS

Install the common tools with:

```sh
bun run setup
```

## Development and validation

```sh
bun run dev:desktop
bun run build:web
bun run build:desktop
bun run check:fast
```

Platform builds:

```sh
bun run build:android
bun run build:ios
bun run package:mac       # package:win and package:linux are also available
```

The Android shell retains `com.Aspenini.MemoryPak`. Slint 1.17 requires Android
API 26, so this is higher than the previous API 24 target. The iOS deployment
target remains iOS 12 with the same bundle identifier.

## Storage and migration

- Desktop saves use the platform application-data directory and atomic
  temporary-file replacement. The previous Memory Pak `ProjectDirs` locations
  are checked automatically.
- Android saves use internal application storage; backup and restore use the
  Storage Access Framework.
- iOS saves use Application Support.
- Web reuses the `memory-pak` IndexedDB database, `state` store, and `persisted`
  key. The PWA precaches the complete Slint/Wasm application and catalog.

Any successfully loaded unversioned save is immediately rewritten as v3.
Malformed saves are not overwritten.

## Stable IDs

Wire IDs remain unchanged:

```text
console:nes
game:nes/super-mario-bros
collectible:skylanders/trigger-happy~2
```

The catalog compiler rejects duplicate IDs. Existing Skylanders duplicates have
explicit `~2` and `~3` source slugs so prior saves continue to resolve.

## Signed updates

Windows and macOS packages use `cargo-packager-updater`. Set the public key at
compile time and the private signing key while packaging:

```text
MEMORY_PAK_UPDATER_PUBKEY
CARGO_PACKAGER_SIGN_PRIVATE_KEY
CARGO_PACKAGER_SIGN_PRIVATE_KEY_PASSWORD
```

Linux remains package/manual-update only. Android and iOS updates are
store-managed. The service worker presents an update prompt for installed PWAs.
