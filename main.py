import sys
import json
import yaml
import os
from typing import List, Dict, Any

from PyQt6.QtWidgets import (
    QApplication, QWidget, QVBoxLayout, QLabel, QScrollArea, QPushButton,
    QHBoxLayout, QLineEdit, QFrame, QSizePolicy, QComboBox, QMainWindow
)
from PyQt6.QtCore import Qt, QTimer
from PyQt6.QtGui import QColor, QPalette, QFont, QIcon


def load_data_file(path: str, fallback: Any) -> Any:
    """Load JSON or YAML file with fallback if file doesn't exist."""
    try:
        if not os.path.exists(path):
            # Create file with fallback data
            if path.endswith('.yaml') or path.endswith('.yml'):
                with open(path, "w", encoding="utf-8") as f:
                    yaml.dump(fallback, f, default_flow_style=False, allow_unicode=True)
            else:
                with open(path, "w", encoding="utf-8") as f:
                    json.dump(fallback, f, indent=2, ensure_ascii=False)
        
        # Load file based on extension
        with open(path, "r", encoding="utf-8") as f:
            if path.endswith('.yaml') or path.endswith('.yml'):
                data = yaml.safe_load(f)
                # Handle YAML structure for consoles
                if path == "consoles.yaml" and data and "consoles" in data:
                    return data["consoles"]
                return data
            else:
                return json.load(f)
    except (json.JSONDecodeError, yaml.YAMLError, IOError) as e:
        print(f"Error loading {path}: {e}")
        return fallback


def save_json(path: str, data: Any) -> None:
    """Save data to JSON file with error handling."""
    try:
        with open(path, "w", encoding="utf-8") as f:
            json.dump(data, f, indent=2, ensure_ascii=False)
    except IOError as e:
        print(f"Error saving {path}: {e}")


class ConsoleCard(QFrame):
    """A card widget representing a console with owned/favorite controls."""
    
    def __init__(self, entry: Dict[str, Any], toggle_theme_callback, refresh_stats_callback):
        super().__init__()
        self.entry = entry
        self.toggle_theme = toggle_theme_callback
        self.refresh_stats = refresh_stats_callback
        
        self.setObjectName("consoleCard")
        self.setFrameStyle(QFrame.Shape.StyledPanel)
        self.setLineWidth(1)
        
        self._setup_ui()
        self._update_button_states()

    def _setup_ui(self):
        """Setup the card UI layout."""
        layout = QHBoxLayout()
        layout.setContentsMargins(15, 10, 15, 10)
        layout.setSpacing(10)

        # Console name and tags
        name_tags = QVBoxLayout()
        name_tags.setSpacing(5)
        
        title = QLabel(self.entry["name"])
        title.setObjectName("consoleName")
        title.setFont(QFont("Segoe UI", 12, QFont.Weight.Bold))
        
        # Ensure all tags are strings and handle any non-string values
        tag_list = self.entry.get("tags", [])
        tag_strings = [str(tag) for tag in tag_list if tag is not None]
        tags = QLabel(", ".join(tag_strings))
        tags.setObjectName("consoleTags")
        tags.setFont(QFont("Segoe UI", 9))
        
        name_tags.addWidget(title)
        name_tags.addWidget(tags)
        layout.addLayout(name_tags)

        layout.addStretch()

        # Owned button
        self.owned_btn = QPushButton()
        self.owned_btn.setCheckable(True)
        self.owned_btn.setObjectName("ownedButton")
        self.owned_btn.clicked.connect(self._toggle_owned)
        layout.addWidget(self.owned_btn)

        # Wishlist button
        self.wishlist_btn = QPushButton()
        self.wishlist_btn.setCheckable(True)
        self.wishlist_btn.setObjectName("wishlistButton")
        self.wishlist_btn.clicked.connect(self._toggle_wishlist)
        layout.addWidget(self.wishlist_btn)

        # Favorite button
        self.favorite_btn = QPushButton()
        self.favorite_btn.setCheckable(True)
        self.favorite_btn.setObjectName("favoriteButton")
        self.favorite_btn.clicked.connect(self._toggle_favorite)
        layout.addWidget(self.favorite_btn)

        self.setLayout(layout)

    def _update_button_states(self):
        """Update button text and checked state based on current data."""
        name = self.entry["name"]
        is_owned = name in user_data["owned"]
        is_favorite = name in user_data["favorite"]
        is_wishlist = name in user_data["wishlist"]
        
        self.owned_btn.setText("✓ Owned" if is_owned else "Mark Owned")
        self.owned_btn.setChecked(is_owned)
        
        # Hide wishlist button if owned, show if not owned
        self.wishlist_btn.setVisible(not is_owned)
        if not is_owned:
            self.wishlist_btn.setText("📋 Wishlist" if is_wishlist else "Add to Wishlist")
            self.wishlist_btn.setChecked(is_wishlist)
        
        self.favorite_btn.setText("♥" if is_favorite else "♡")
        self.favorite_btn.setChecked(is_favorite)

    def _toggle_owned(self):
        """Toggle the owned status of this console."""
        name = self.entry["name"]
        if name in user_data["owned"]:
            user_data["owned"].remove(name)
        else:
            user_data["owned"].append(name)
            # Remove from wishlist if now owned
            if name in user_data["wishlist"]:
                user_data["wishlist"].remove(name)
        
        save_json("user_data.json", user_data)
        self._update_button_states()
        self.refresh_stats()

    def _toggle_wishlist(self):
        """Toggle the wishlist status of this console."""
        name = self.entry["name"]
        if name in user_data["wishlist"]:
            user_data["wishlist"].remove(name)
        else:
            user_data["wishlist"].append(name)
        
        save_json("user_data.json", user_data)
        self._update_button_states()
        self.refresh_stats()

    def _toggle_favorite(self):
        """Toggle the favorite status of this console."""
        name = self.entry["name"]
        if name in user_data["favorite"]:
            user_data["favorite"].remove(name)
        else:
            user_data["favorite"].append(name)
        
        save_json("user_data.json", user_data)
        self._update_button_states()
        self.refresh_stats()


class MemoryPak(QMainWindow):
    """Main application window for Memory Pak."""
    
    def __init__(self):
        super().__init__()
        self.setWindowTitle("Memory Pak - Console Collection Manager")
        self.setGeometry(100, 100, 1000, 800)
        self.setMinimumSize(800, 600)
        
        self._setup_ui()
        self.refresh_cards()

    def _setup_ui(self):
        """Setup the main UI layout."""
        central_widget = QWidget()
        self.setCentralWidget(central_widget)
        
        layout = QVBoxLayout(central_widget)
        layout.setSpacing(15)
        layout.setContentsMargins(20, 20, 20, 20)

        # Top bar with search and controls
        self._setup_top_bar(layout)
        
        # Stats display
        self._setup_stats(layout)
        
        # Console cards area
        self._setup_cards_area(layout)

    def _setup_top_bar(self, parent_layout):
        """Setup the top bar with search, theme toggle, and sort controls."""
        top_bar = QHBoxLayout()
        top_bar.setSpacing(10)

        # Search box
        self.search_box = QLineEdit()
        self.search_box.setPlaceholderText("Search consoles or tags...")
        self.search_box.setObjectName("searchBox")
        self.search_box.textChanged.connect(self._on_search_changed)
        top_bar.addWidget(self.search_box)

        # Theme toggle button
        self.theme_btn = QPushButton("🌙 Dark Mode")
        self.theme_btn.setObjectName("themeButton")
        self.theme_btn.clicked.connect(self._toggle_theme)
        top_bar.addWidget(self.theme_btn)

        # Filter dropdown
        self.filter_box = QComboBox()
        self.filter_box.addItems([
            "All Consoles",
            "Home Consoles",
            "Handhelds",
            "VR Headsets",
            "PC Gaming Handhelds",
            "Retro Consoles",
            "Modern Consoles (8th-9th Gen)",
            "Wishlist"
        ])
        self.filter_box.setObjectName("filterBox")
        self.filter_box.currentIndexChanged.connect(self._on_filter_changed)
        top_bar.addWidget(self.filter_box)

        # Sort dropdown
        self.sort_box = QComboBox()
        self.sort_box.addItems([
            "A-Z", 
            "Owned First", 
            "Wishlist First",
            "Favorites First",
            "Generation (Oldest First)",
            "Generation (Newest First)",
            "Type (Consoles First)",
            "Type (Handhelds First)"
        ])
        self.sort_box.setObjectName("sortBox")
        self.sort_box.currentIndexChanged.connect(self._on_sort_changed)
        top_bar.addWidget(self.sort_box)

        parent_layout.addLayout(top_bar)

    def _setup_stats(self, parent_layout):
        """Setup the statistics display."""
        self.stats_label = QLabel()
        self.stats_label.setObjectName("statsLabel")
        self.stats_label.setAlignment(Qt.AlignmentFlag.AlignCenter)
        self.stats_label.setFont(QFont("Segoe UI", 10))
        parent_layout.addWidget(self.stats_label)

    def _setup_cards_area(self, parent_layout):
        """Setup the scrollable cards area."""
        self.scroll_area = QScrollArea()
        self.scroll_area.setWidgetResizable(True)
        self.scroll_area.setObjectName("scrollArea")
        self.scroll_area.setHorizontalScrollBarPolicy(Qt.ScrollBarPolicy.ScrollBarAlwaysOff)
        
        self.cards_container = QWidget()
        self.cards_layout = QVBoxLayout(self.cards_container)
        self.cards_layout.setSpacing(8)
        self.cards_layout.setContentsMargins(0, 0, 0, 0)
        
        self.scroll_area.setWidget(self.cards_container)
        parent_layout.addWidget(self.scroll_area)

    def _on_search_changed(self):
        """Handle search text changes with debouncing."""
        if hasattr(self, '_search_timer'):
            self._search_timer.stop()
        else:
            self._search_timer = QTimer()
            self._search_timer.setSingleShot(True)
            self._search_timer.timeout.connect(self.refresh_cards)
        
        self._search_timer.start(300)  # 300ms delay

    def _on_sort_changed(self):
        """Handle sort selection changes."""
        self.refresh_cards()

    def _on_filter_changed(self):
        """Handle filter selection changes."""
        self.refresh_cards()

    def refresh_cards(self):
        """Refresh the console cards display."""
        # Clear existing cards
        for i in reversed(range(self.cards_layout.count())):
            widget = self.cards_layout.itemAt(i).widget()
            if widget:
                widget.setParent(None)

        query = self.search_box.text().lower().strip()
        entries = list(consoles)

        # Apply filter
        filter_mode = self.filter_box.currentText()
        if filter_mode == "Home Consoles":
            entries = [c for c in entries if "console" in [str(tag) for tag in c.get("tags", [])] and "handheld" not in [str(tag) for tag in c.get("tags", [])]]
        elif filter_mode == "Handhelds":
            entries = [c for c in entries if "handheld" in [str(tag) for tag in c.get("tags", [])]]
        elif filter_mode == "VR Headsets":
            entries = [c for c in entries if "vr" in [str(tag) for tag in c.get("tags", [])]]
        elif filter_mode == "PC Gaming Handhelds":
            entries = [c for c in entries if "pc-gaming" in [str(tag) for tag in c.get("tags", [])]]
        elif filter_mode == "Retro Consoles":
            entries = [c for c in entries if "retro" in [str(tag) for tag in c.get("tags", [])]]
        elif filter_mode == "Modern Consoles (8th-9th Gen)":
            entries = [c for c in entries if any(str(tag) in ["8th-gen", "9th-gen"] for tag in c.get("tags", []) if tag is not None)]
        elif filter_mode == "Wishlist":
            entries = [c for c in entries if c["name"] in user_data["wishlist"]]

        # Apply search filter
        if query:
            entries = [
                c for c in entries
                if query in c["name"].lower() or 
                   any(query in str(tag).lower() for tag in c.get("tags", []) if tag is not None)
            ]

        # Sort entries
        sort_mode = self.sort_box.currentText()
        if sort_mode == "Owned First":
            entries.sort(key=lambda x: (x["name"] not in user_data["owned"], x["name"]))
        elif sort_mode == "Wishlist First":
            entries.sort(key=lambda x: (x["name"] not in user_data["wishlist"], x["name"]))
        elif sort_mode == "Favorites First":
            entries.sort(key=lambda x: (x["name"] not in user_data["favorite"], x["name"]))
        elif sort_mode == "Generation (Oldest First)":
            entries.sort(key=lambda x: (_get_generation_order(x), x["name"]))
        elif sort_mode == "Generation (Newest First)":
            entries.sort(key=lambda x: (-_get_generation_order(x), x["name"]))
        elif sort_mode == "Type (Consoles First)":
            entries.sort(key=lambda x: (_is_handheld(x), x["name"]))
        elif sort_mode == "Type (Handhelds First)":
            entries.sort(key=lambda x: (not _is_handheld(x), x["name"]))
        else:  # A-Z
            entries.sort(key=lambda x: x["name"])

        # Create and add cards
        for entry in entries:
            card = ConsoleCard(entry, self._toggle_theme, self.refresh_stats)
            self.cards_layout.addWidget(card)

        # Add stretch to push cards to top
        self.cards_layout.addStretch()
        
        self.refresh_stats()

    def refresh_stats(self):
        """Update the statistics display."""
        total = len(consoles)
        owned = len(user_data["owned"])
        wishlist = len(user_data["wishlist"])
        favorited = len(user_data["favorite"])
        
        stats_text = f"🎮 Total: {total} | ✔ Owned: {owned} | 📋 Wishlist: {wishlist} | ❤️ Favorites: {favorited}"
        if total > 0:
            owned_percent = (owned / total) * 100
            stats_text += f" | 📊 {owned_percent:.1f}% Complete"
        
        self.stats_label.setText(stats_text)

    def _toggle_theme(self):
        """Toggle between light and dark themes."""
        current_theme = settings.get("theme", "dark")
        new_theme = "light" if current_theme == "dark" else "dark"
        
        settings["theme"] = new_theme
        save_json("settings.json", settings)
        
        self.theme_btn.setText("🌙 Dark Mode" if new_theme == "dark" else "☀️ Light Mode")
        apply_theme(QApplication.instance(), new_theme)


def _get_generation_order(entry: Dict[str, Any]) -> int:
    """Get generation order for sorting (lower = older)."""
    tags = [str(tag) for tag in entry.get("tags", []) if tag is not None]
    
    # Generation mapping
    generation_map = {
        "retro": 1,
        "8-bit": 2,
        "16-bit": 3,
        "5th-gen": 4,
        "6th-gen": 5,
        "7th-gen": 6,
        "8th-gen": 7,
        "9th-gen": 8
    }
    
    for tag in tags:
        if tag in generation_map:
            return generation_map[tag]
    
    # Default to end if no generation tag found
    return 999


def _is_handheld(entry: Dict[str, Any]) -> bool:
    """Check if console is a handheld."""
    return "handheld" in [str(tag) for tag in entry.get("tags", []) if tag is not None]


def apply_theme(app: QApplication, theme: str):
    """Apply the specified theme to the application."""
    palette = QPalette()
    
    if theme == "dark":
        # Dark theme colors
        palette.setColor(QPalette.ColorRole.Window, QColor(53, 53, 53))
        palette.setColor(QPalette.ColorRole.WindowText, QColor(255, 255, 255))
        palette.setColor(QPalette.ColorRole.Base, QColor(25, 25, 25))
        palette.setColor(QPalette.ColorRole.AlternateBase, QColor(53, 53, 53))
        palette.setColor(QPalette.ColorRole.ToolTipBase, QColor(255, 255, 255))
        palette.setColor(QPalette.ColorRole.ToolTipText, QColor(255, 255, 255))
        palette.setColor(QPalette.ColorRole.Text, QColor(255, 255, 255))
        palette.setColor(QPalette.ColorRole.Button, QColor(53, 53, 53))
        palette.setColor(QPalette.ColorRole.ButtonText, QColor(255, 255, 255))
        palette.setColor(QPalette.ColorRole.BrightText, QColor(255, 0, 0))
        palette.setColor(QPalette.ColorRole.Link, QColor(42, 130, 218))
        palette.setColor(QPalette.ColorRole.Highlight, QColor(42, 130, 218))
        palette.setColor(QPalette.ColorRole.HighlightedText, QColor(255, 255, 255))
    else:
        # Light theme colors
        palette.setColor(QPalette.ColorRole.Window, QColor(240, 240, 240))
        palette.setColor(QPalette.ColorRole.WindowText, QColor(0, 0, 0))
        palette.setColor(QPalette.ColorRole.Base, QColor(255, 255, 255))
        palette.setColor(QPalette.ColorRole.AlternateBase, QColor(245, 245, 245))
        palette.setColor(QPalette.ColorRole.ToolTipBase, QColor(255, 255, 255))
        palette.setColor(QPalette.ColorRole.ToolTipText, QColor(0, 0, 0))
        palette.setColor(QPalette.ColorRole.Text, QColor(0, 0, 0))
        palette.setColor(QPalette.ColorRole.Button, QColor(240, 240, 240))
        palette.setColor(QPalette.ColorRole.ButtonText, QColor(0, 0, 0))
        palette.setColor(QPalette.ColorRole.BrightText, QColor(255, 0, 0))
        palette.setColor(QPalette.ColorRole.Link, QColor(0, 0, 255))
        palette.setColor(QPalette.ColorRole.Highlight, QColor(42, 130, 218))
        palette.setColor(QPalette.ColorRole.HighlightedText, QColor(255, 255, 255))
    
    app.setPalette(palette)


def main():
    """Main application entry point."""
    # Load data
    global consoles, user_data, settings
    consoles = load_data_file("consoles.yaml", [])
    user_data = load_data_file("user_data.json", {"owned": [], "wishlist": [], "favorite": []})
    settings = load_data_file("settings.json", {"theme": "dark"})
    
    # Create application
    app = QApplication(sys.argv)
    app.setApplicationName("Memory Pak")
    app.setApplicationVersion("1.0")
    
    # Apply theme
    apply_theme(app, settings.get("theme", "dark"))
    
    # Create and show main window
    window = MemoryPak()
    window.show()
    
    # Start event loop
    sys.exit(app.exec())


if __name__ == "__main__":
    main()
