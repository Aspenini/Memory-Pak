set shell := ["sh", "-cu"]

default:
    just --list

# Install Rust and frontend tools used by the new stack.
install-tools:
    rustup target add wasm32-unknown-unknown
    cargo install wasm-pack tauri-cli
    bun install --cwd frontend

# Copy the canonical icon set in icons/web into the PWA public folder.
# Run this whenever icons/web/ changes so the PWA bundle stays in sync.
sync-icons:
    rm -f frontend/public/icons/*.png frontend/public/icons/*.ico
    cp icons/web/apple-touch-icon.png frontend/public/icons/
    cp icons/web/favicon.ico frontend/public/icons/
    cp icons/web/icon-192.png frontend/public/icons/
    cp icons/web/icon-192-maskable.png frontend/public/icons/
    cp icons/web/icon-512.png frontend/public/icons/
    cp icons/web/icon-512-maskable.png frontend/public/icons/

# Rust checks
fmt:
    cargo fmt --all

clippy:
    cargo clippy --workspace --all-targets -- -D warnings

test:
    cargo test --workspace

check:
    cargo check --workspace

check-wasm:
    cargo check -p memory_pak_wasm --target wasm32-unknown-unknown

# Frontend
frontend-dev:
    bun run --cwd frontend dev

frontend-build:
    bun run --cwd frontend build

frontend-test:
    bun run --cwd frontend test

frontend-e2e:
    bun run --cwd frontend e2e

# Tauri desktop
tauri-dev:
    cargo tauri dev

tauri-build:
    cargo tauri build

# Tauri mobile
android-init:
    cargo tauri android init

android-dev:
    cargo tauri android dev

android-build:
    cargo tauri android build

ios-init:
    cargo tauri ios init

ios-dev:
    cargo tauri ios dev

ios-build:
    cargo tauri ios build

all-checks: fmt clippy test check-wasm frontend-test frontend-build
