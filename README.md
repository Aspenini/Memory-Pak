# Memory-Pak

A comprehensive console and game collection manager built with PyQt6.

## Features

### 🎮 Console Management
- Track owned, wishlisted, and favorite consoles
- Search and filter by console type (Home, Handheld, VR, etc.)
- Sort by generation, type, or alphabetical order
- Statistics and completion tracking

### 🎯 Game Management
- Browse games by console (NES, SNES, PS1, etc.)
- Mark games as owned, wishlisted, or favorites
- Search games by name, developer, or publisher
- Comprehensive game databases for 9+ consoles

### 🎨 User Interface
- Modern tabbed interface
- Dark/Light theme support
- Responsive design with scrollable lists
- Clean, intuitive controls

## Build Instructions

### Prerequisites
- Python 3.8 or higher
- Windows 10/11

### Build
```bash
# Run the build script (always does a clean build)
build.bat
```

### Manual Build
```bash
# Install dependencies
pip install -r requirements.txt

# Build with PyInstaller
pyinstaller --onefile --windowed --name "Memory-Pak" main.py

# Copy data files
copy consoles.yaml dist\
xcopy games dist\games\ /E /I /Y
```

## Project Structure

```
Memory-Pak/
├── main.py              # Main application
├── consoles.yaml        # Console database
├── games/               # Game databases
│   ├── nes.yaml
│   ├── snes.yaml
│   ├── ps1.yaml
│   └── ...
├── icons/               # Console icons
├── build.bat           # Build script
└── requirements.txt    # Python dependencies
```

## Data Files

### User Data (Auto-created)
- `user_data.json` - Unified collection data (consoles and games)
- `settings.json` - Application settings

### Static Data
- `consoles.yaml` - Console definitions and tags
- `games/*.yaml` - Game databases by console

## Available Consoles

### Home Consoles
- NES, SNES, N64, GameCube, Wii, Wii U, Switch
- PlayStation 1-5, Xbox, Xbox 360, Xbox One, Xbox Series X/S

### Handhelds
- Game Boy, GBA, DS, 3DS, PSP, PS Vita
- Steam Deck, ROG Ally, Legion Go

### VR Headsets
- Oculus Quest, Meta Quest 2/3/Pro
- Valve Index, HTC Vive

## Game Databases

- **NES**: 437 games
- **SNES**: 591 games
- **N64**: 278 games
- **GameCube**: 442 games
- **Wii**: 569 games
- **Wii U**: 136 games
- **DS**: 564 games
- **3DS**: 546 games
- **PS1**: 580 games

## Development

### Running from Source
```bash
python main.py
```

### Dependencies
- PyQt6 - GUI framework
- PyYAML - YAML file parsing
- PyInstaller - Application packaging

## License

This project is open source. Feel free to modify and distribute. 