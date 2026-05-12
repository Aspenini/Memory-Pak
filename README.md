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
├── crates/
│   ├── memory_pak_core/   # shared Rust data model, queries, state reducer, import/export
│   └── memory_pak_wasm/   # wasm-bindgen adapter for the browser/PWA target
├── frontend/              # Svelte 5 + TypeScript + Vite app
├── src-tauri/             # Tauri 2 desktop/mobile shell and commands
├── database/              # `consoles.json`, `games/*.json`, `collectibles/*.json`
├── icons/                 # platform icons reused by Tauri and PWA
└── site/                  # GitHub Pages landing page; deploy copies frontend/dist to site/app
```

## Requirements

- Rust stable (pinned via `rust-toolchain.toml`)
- Bun 1.3+
- Tauri platform prerequisites for desktop/mobile builds (Xcode CLT on macOS, WebView2 on Windows, `webkit2gtk` etc. on Linux)

Install the rest of the tooling (`wasm-pack`, `tauri-cli`, the WASM target, and frontend deps) in one shot:

```bash
bun run install-tools
```

## Development

All commands are exposed as Bun scripts in the root `package.json`. Inspect that file to see every available script.

```bash
bun run test               # cargo test --workspace
bun run check:wasm         # check WASM adapter against wasm32-unknown-unknown
bun run frontend:dev       # Svelte/Vite PWA development server
bun run frontend:build     # production PWA build
bun run frontend:e2e       # Playwright end-to-end tests
bun run tauri:dev          # Tauri desktop app
bun run all-checks         # fmt + clippy + test + check:wasm + frontend test/build
```

Icons under `icons/web/` are the canonical PWA icon source. Vite serves them under `/icons/...` in dev and emits them to `dist/icons/...` at build time, so there is no separate copy to keep in sync.

Tauri mobile entrypoints are scaffolded through the standard Tauri CLI:

```bash
bun run android:init
bun run android:dev
bun run android:build

bun run ios:init
bun run ios:dev
bun run ios:build
```

## User Data Storage

- **Desktop / mobile**: a single `state.json` under the OS data directory (`ProjectDirs::data_dir()/state.json`), written atomically via a temp file + rename.
- **Web / PWA**: a single IndexedDB record in the `memory-pak` database, written debounced to coalesce rapid toggles.

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
