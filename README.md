# Memory Pak

A cross-platform game collection tracker built with Rust, egui, and eframe. Memory Pak allows you to track which consoles and games you own, have favorited, or want (wishlist).

[![Crates.io](https://img.shields.io/crates/v/memory-pak?style=for-the-badge)](https://crates.io/crates/memory-pak)
[![Crates.io Downloads](https://img.shields.io/crates/d/memory-pak?style=for-the-badge)](https://crates.io/crates/memory-pak)
[![GitHub Release](https://img.shields.io/github/v/release/Aspenini/Memory-Pak?style=for-the-badge)](https://github.com/Aspenini/Memory-Pak/releases)
[![GitHub Downloads](https://img.shields.io/github/downloads/Aspenini/Memory-Pak/total?style=for-the-badge)](https://github.com/Aspenini/Memory-Pak/releases)

## Features

- **Two Main Tabs**: Consoles and Games (both with pagination)
- **Hardcoded Console List**: Includes popular consoles from Nintendo, Sega, Sony, and Microsoft
- **Console States**: Mark consoles as Owned, Favorite, or Wishlist, with notes
- **Game States**: Mark games as Owned, Favorite, or Wishlist, with notes
- **Embedded Game Database**: All game data is compiled directly into the binary using `include_dir!()` at runtime
- **Stable ID System**: Games use content-based IDs (console + title hash), allowing database updates without breaking saved states
- **Cross-Console Search**: View all games across all consoles with console badges, or filter by specific console
- **Pagination**: 
  - Games tab: 50 games per page
  - Consoles tab: 20 consoles per page
- **State Persistence**: Saves user data to platform-specific directories (desktop/mobile) or localStorage (web)
- **Sorting Options**: Sort games by title, release year, or status
- **Filtering Options**: Filter by All, Owned, Favorites, Wishlist, or Not Owned
- **Search**: Search games by title across all consoles
- **Import/Export**: Export your entire collection state (consoles + games) to JSON and import it back
- **Single Binary**: Everything is embedded, no external files needed

## Project Structure

```
Memory-Pak/
├── Cargo.toml          # Project configuration
├── src/
│   ├── main.rs        # Main application entry point
│   ├── console_data.rs # Hardcoded console definitions
│   ├── game_data.rs   # Embedded game data loading (runtime JSON parsing)
│   ├── persistence.rs # Save/load user state (native & web)
│   └── ui.rs          # UI rendering functions
└── database/          # Game JSON files (embedded at compile time)
    ├── nes.json
    ├── snes.json
    ├── n64.json
    ├── gameboyadvance.json
    └── ... (one JSON file per console)
```

## Building

### Standard Build

```bash
cargo build --release
```

Binary output: `target/release/memory_pak[.exe]`

### Web Build (WASM)

```bash
# Install Trunk (one-time)
cargo install trunk
rustup target add wasm32-unknown-unknown

# Development with hot reload → opens at http://127.0.0.1:8080
trunk serve

# Production build → output in dist/
trunk build --release
```

### Platform Installers

**Windows (MSI):**
```bash
cargo install cargo-wix          # One-time install
cargo wix                        # Build MSI
```
Output: `target/wix/Memory-Pak-0.1.4-x86_64.msi`  
Requires: [WiX Toolset v3.11+](https://wixtoolset.org/)

**macOS (.app):**
```bash
cargo install cargo-bundle       # One-time install
cargo bundle --release
```
Output: `target/release/bundle/osx/Memory Pak.app`

**Linux (.deb):**
```bash
cargo install cargo-deb          # One-time install
cargo deb
```
Output: `target/debian/memory_pak_*.deb`

### WebAssembly

```bash
cargo install wasm-pack          # One-time install
wasm-pack build --target web --out-dir pkg
```

## User Data Storage

- **Desktop/Mobile**: Stored in platform-specific user data directories:
  - Windows: `%APPDATA%\com\memorypak\memory_pak\state\`
  - macOS: `~/Library/Application Support/com.memorypak.memory_pak/state/`
  - Linux: `~/.local/share/com/memorypak/memory_pak/state/`

- **Web**: Stored in browser localStorage with keys like `memory_pak_state_{console_id}`

## Export Format

The export file contains all your console and game states in this format:

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

**Note:** Game IDs use the stable ID format (`{console_id}-{hash}`), ensuring compatibility across database updates.