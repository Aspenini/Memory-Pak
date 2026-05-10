# Memory Pak

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
just install-tools
```

## Development

```bash
just test             # cargo test --workspace
just check-wasm       # check WASM adapter against wasm32-unknown-unknown
just frontend-dev     # Svelte/Vite PWA development server
just frontend-build   # production PWA build
just tauri-dev        # Tauri desktop app
```

Tauri mobile entrypoints are scaffolded through the standard Tauri CLI:

```bash
just android-init
just android-dev
just android-build

just ios-init
just ios-dev
just ios-build
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
