# Memory Pak

A cross-platform game collection tracker built with Rust, egui, and eframe. Memory Pak allows you to track which consoles and games you own, have favorited, or want (wishlist).

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

### Desktop (Windows, macOS, Linux)

```bash
cargo build --release
```

The executable will be in `target/release/memory-pak` (or `memory-pak.exe` on Windows).

### WebAssembly

```bash
# Install wasm-pack if you haven't already
cargo install wasm-pack

# Build for web
wasm-pack build --target web --out-dir pkg
```

### Android/iOS

For mobile platforms, you'll need to set up the appropriate toolchains:

- **Android**: Use `cargo-apk` or follow egui's Android setup guide
- **iOS**: Use `cargo-lipo` or follow egui's iOS setup guide

## Adding Game Data

To add games for a console, create a JSON file in `database/` named after the console ID (e.g., `ps5.json`, `nes.json`). The format is:

```json
[
  {
    "title": "Marvel's Spider-Man 2",
    "developer": "Insomniac Games",
    "publisher": "Sony Interactive Entertainment",
    "release_date": "2023-10-20"
  }
]
```

**Fields:**
- `title` (required): Game title
- `publisher` (required): Publisher name
- `developer` (optional): Developer name
- `release_date` (optional): ISO format date (YYYY-MM-DD), extracted year is used for display
- Additional regional release dates (`jp_release`, `na_release`, `pal_release`) may be included but are not used

**Stable IDs:**
- Games automatically get stable IDs based on console ID + title hash
- This means you can add, remove, or reorder games in JSON files without breaking existing saved states
- Console ID is derived from the JSON filename (e.g., `nes.json` → console ID `"nes"`)

The game data will be embedded into the binary at compile time using `include_dir!()`.

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

## License

This project is provided as-is for personal use.

