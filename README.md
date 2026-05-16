# Memory Pak

[![CI](https://github.com/Aspenini/Memory-Pak/actions/workflows/ci.yml/badge.svg)](https://github.com/Aspenini/Memory-Pak/actions/workflows/ci.yml)
[![Deploy Website](https://github.com/Aspenini/Memory-Pak/actions/workflows/deploy.yml/badge.svg)](https://github.com/Aspenini/Memory-Pak/actions/workflows/deploy.yml)
[![GitHub Release](https://img.shields.io/github/v/release/Aspenini/Memory-Pak?label=release)](https://github.com/Aspenini/Memory-Pak/releases/latest)
[![GitHub Release Downloads](https://img.shields.io/github/downloads/Aspenini/Memory-Pak/total?label=release%20downloads)](https://github.com/Aspenini/Memory-Pak/releases)
[![License](https://img.shields.io/github/license/Aspenini/Memory-Pak)](LICENSE)

A cross-platform game collection tracker built with Rust, Tauri 2, Svelte, and WebAssembly. Memory Pak tracks consoles, games, and toy-to-life collectibles (LEGO Dimensions, Skylanders, and more) across owned, favorite, wishlist, and notes states.

## Features

- Desktop and mobile app shells through Tauri 2
- Static web/PWA build using the same Svelte frontend and Rust core compiled to WASM
- Embedded catalog precompiled at build time into a single binary blob (`postcard`)
- Deterministic slug-based entry IDs (`game:nes/super-mario-bros`, `collectible:legodimensions/batman`, etc.)
- Unified Collectibles tab spanning every toy-to-life line in `database/collectibles/`
- Cross-console search, sorting, filtering, and virtualized long lists
- JSON import/export at schema version `2.0`

## Project Structure

```text
Memory-Pak/
|-- crates/
|   |-- memory_pak_core/   # shared Rust data model, queries, state reducer, import/export
|   `-- memory_pak_wasm/   # wasm-bindgen adapter for the browser/PWA target
|-- frontend/              # Svelte 5 + TypeScript + Vite app
|-- src-tauri/             # Tauri 2 desktop/mobile shell and commands
|-- database/              # `consoles.json`, `games/*.json`, `collectibles/*.json`
|-- icons/                 # platform icons reused by Tauri and PWA
`-- site/                  # GitHub Pages landing page; deploy copies frontend/dist to site/app
```

## Requirements

- Rust 1.94.0 (pinned via `rust-toolchain.toml`)
- Bun 1.3.13
- `wasm-pack` 0.14.0
- Tauri CLI 2.11.1
- Tauri platform prerequisites for desktop/mobile builds (Xcode CLT on macOS, WebView2 on Windows, `webkit2gtk` etc. on Linux)

Install the tooling (`wasm-pack`, `tauri-cli`, the WASM target, and frontend deps) in one shot:

```bash
bun run setup
```

## Development

Root scripts are grouped by task:

```bash
bun run dev:web            # Svelte/Vite PWA development server
bun run dev:desktop        # Tauri desktop app
bun run dev:android        # Tauri Android app
bun run build:web          # production PWA build
bun run build:desktop      # desktop integration build without installers
bun run build:android      # Android package build
bun run package:desktop    # desktop installer/package build
bun run package:win        # Windows NSIS/MSI bundles
bun run package:mac        # macOS DMG bundle
bun run package:linux      # Linux AppImage/deb bundles
bun run check:fast         # fmt + clippy + Rust tests + WASM + frontend checks/build
bun run check:ci           # check:fast + Playwright + desktop smoke build
```

Icons under `icons/web/` are the canonical PWA icon source. Vite serves them under `/icons/...` in dev and emits them to `dist/icons/...` at build time, so there is no separate copy to keep in sync.

Generated WASM bindings are written to `frontend/generated/wasm/` and ignored by git. Frontend check/build/dev scripts generate them before TypeScript or Vite runs.

Frontend-only scripts live in `frontend/package.json`; run them directly with `bun run --cwd frontend <script>` when needed.

Tauri mobile entrypoints are scaffolded through the standard Tauri CLI:

```bash
bun run android:init
bun run dev:android
bun run build:android

bun run ios:init
```

## User Data Storage

- **Desktop / mobile**: a single `state.json` under the OS data directory (`ProjectDirs::data_dir()/state.json`), written atomically via a temp file + rename.
- **Web / PWA**: a single IndexedDB record in the `memory-pak` database, written debounced to coalesce rapid toggles.

## Releases and Updates

Normal CI validates the project only. The manual **Package Artifacts** workflow builds Windows, macOS, and Linux bundles, creates updater signatures, and uploads workflow artifacts plus `latest.json` and `checksums.sha256`. It does not publish a GitHub release; attach those artifacts to the chosen release manually.

Desktop self-updates use the Tauri updater and require these repository secrets for the manual package workflow:

```text
TAURI_SIGNING_PRIVATE_KEY
TAURI_SIGNING_PRIVATE_KEY_PASSWORD   # optional if your key has no password
TAURI_UPDATER_PUBKEY
```

The generated `latest.json` is intended to be attached to the same GitHub release as the bundles. Linux auto-update targets AppImage installs; `.deb` remains a manual installer. The web app prompts when the PWA service worker sees a newer build. Android uses Google Play's in-app update flow when installed from a Play track and falls back to the store listing otherwise.

## Export Format

```json
{
  "version": "2.0",
  "exportedAt": "2024-01-01T00:00:00Z",
  "entries": [
    {
      "id": "console:nes",
      "owned": true,
      "favorite": false,
      "wishlist": false,
      "notes": "My original NES"
    },
    {
      "id": "game:nes/super-mario-bros",
      "owned": true,
      "favorite": true,
      "wishlist": false,
      "notes": ""
    }
  ]
}
```
