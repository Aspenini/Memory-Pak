# Memory Pak

[![CI](https://github.com/Aspenini/Memory-Pak/actions/workflows/ci.yml/badge.svg)](https://github.com/Aspenini/Memory-Pak/actions/workflows/ci.yml)
[![Deploy Website](https://github.com/Aspenini/Memory-Pak/actions/workflows/deploy.yml/badge.svg)](https://github.com/Aspenini/Memory-Pak/actions/workflows/deploy.yml)
[![GitHub Release](https://img.shields.io/github/v/release/Aspenini/Memory-Pak?label=release)](https://github.com/Aspenini/Memory-Pak/releases/latest)
[![GitHub Release Downloads](https://img.shields.io/github/downloads/Aspenini/Memory-Pak/total?label=release%20downloads)](https://github.com/Aspenini/Memory-Pak/releases)
[![crates.io](https://img.shields.io/crates/v/Memory-Pak?label=crates.io)](https://crates.io/crates/Memory-Pak)
[![crates.io Downloads](https://img.shields.io/crates/d/Memory-Pak?label=crates.io%20downloads)](https://crates.io/crates/Memory-Pak)
[![License](https://img.shields.io/github/license/Aspenini/Memory-Pak)](LICENSE)

A cross-platform game collection tracker built with Rust, Tauri 2, Svelte, and WebAssembly. Memory Pak tracks consoles, games, LEGO Dimensions figures, and Skylanders across owned, favorite, wishlist, and notes states.

## Features

- Desktop and mobile app shells through Tauri 2
- Static web/PWA build using the same Svelte frontend and Rust core compiled to WASM
- Embedded game and collector databases
- Stable content-based game IDs so database updates do not break saved states
- Cross-console search, sorting, filtering, and virtualized long lists
- Compatible JSON import/export format, still versioned as `1.0`
- Saved-data compatibility with the previous Memory Pak state files and web localStorage keys

## Project Structure

```text
Memory-Pak/
├── crates/
│   ├── memory_pak_core/   # shared Rust data model, queries, state reducer, import/export
│   └── memory_pak_wasm/   # wasm-bindgen adapter for the browser/PWA target
├── frontend/              # Svelte 5 + TypeScript + Vite app
├── src-tauri/             # Tauri 2 desktop/mobile shell and commands
├── database/              # embedded JSON databases
├── icons/                 # platform icons reused by Tauri and PWA
└── site/                  # GitHub Pages landing page; deploy copies frontend/dist to site/app
```

## Requirements

- Rust stable
- Bun 1.3+
- Tauri platform prerequisites for desktop/mobile builds (Xcode CLT on macOS, WebView2 on Windows, `webkit2gtk` etc. on Linux)

Install the rest of the tooling (`wasm-pack`, `tauri-cli`, the WASM target,
and frontend deps) in one shot:

```bash
bun run install-tools
```

## Development

All commands are exposed as Bun scripts in the root `package.json`. Run
`bun run --list` to see every available script.

```bash
bun run test               # cargo test --workspace
bun run check:wasm         # check WASM adapter against wasm32-unknown-unknown
bun run frontend:dev       # Svelte/Vite PWA development server
bun run frontend:build     # production PWA build
bun run tauri:dev          # Tauri desktop app
bun run all-checks         # fmt + clippy + test + check:wasm + frontend test/build
```

Icons under `icons/web/` are the canonical PWA icon source. Vite serves
them under `/icons/...` in dev and emits them to `dist/icons/...` at
build time, so there is no separate copy to keep in sync.

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

Desktop and mobile use the existing Memory Pak data directory and state files:

- `consoles.json`
- `{console_id}.json`
- `lego_dimensions.json`
- `skylanders.json`

The web/PWA target migrates and preserves the previous localStorage keys:

- `memory_pak_console_states`
- `memory_pak_state_{console_id}`
- `memory_pak_lego_dimensions_states`
- `memory_pak_skylanders_states`

It also writes a versioned snapshot key, `memory_pak_state_v2`, for faster startup.

## Export Format

The export schema remains compatible with previous releases:

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
      "notes": "My original NES"
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
  ],
  "lego_dimensions_states": [],
  "skylanders_states": []
}
```
