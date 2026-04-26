set shell := ["powershell.exe", "-NoLogo", "-NoProfile", "-Command"]

app := "memory_pak"
android_targets := "aarch64-linux-android armv7-linux-androideabi x86_64-linux-android"
android_keystore := ".android/memory-pak-release.keystore"

default:
    just --list

# Shared quality checks
fmt:
    cargo fmt

clippy:
    cargo clippy --all-targets --all-features -- -D warnings

test:
    cargo test --all-targets --all-features

all-checks: fmt clippy test check check-web web-build

# Desktop
check:
    cargo check

build:
    cargo build --release

run:
    cargo run

# Windows
windows-msi:
    cargo wix

# macOS
macos-app:
    cargo bundle --release

# Linux
linux-deb:
    cargo deb

# Web
install-web-tools:
    rustup target add wasm32-unknown-unknown
    cargo install trunk

check-web:
    cargo check --target wasm32-unknown-unknown

web-build:
    trunk build --release

web-serve:
    trunk serve

# Android
install-android-tools:
    rustup target add {{android_targets}}
    cargo install cargo-apk

android-keystore:
    New-Item -ItemType Directory -Force .android | Out-Null; keytool -genkeypair -v -keystore "{{android_keystore}}" -alias memory-pak -keyalg RSA -keysize 4096 -validity 10000 -dname "CN=Memory Pak, O=Memory Pak, C=US"

android-debug-build:
    cargo apk build --lib

android-build:
    if (!(Test-Path "{{android_keystore}}")) { throw "Missing {{android_keystore}}. Run 'just android-keystore' once and keep that file safe." }; $env:CARGO_APK_RELEASE_KEYSTORE = (Resolve-Path "{{android_keystore}}").Path; $secure = Read-Host "Release keystore password" -AsSecureString; $ptr = [Runtime.InteropServices.Marshal]::SecureStringToBSTR($secure); try { $env:CARGO_APK_RELEASE_KEYSTORE_PASSWORD = [Runtime.InteropServices.Marshal]::PtrToStringBSTR($ptr); cargo apk build --lib --release } finally { [Runtime.InteropServices.Marshal]::ZeroFreeBSTR($ptr); Remove-Item Env:CARGO_APK_RELEASE_KEYSTORE_PASSWORD -ErrorAction SilentlyContinue }

android-run:
    if (!(Test-Path "{{android_keystore}}")) { throw "Missing {{android_keystore}}. Run 'just android-keystore' once and keep that file safe." }; $env:CARGO_APK_RELEASE_KEYSTORE = (Resolve-Path "{{android_keystore}}").Path; $secure = Read-Host "Release keystore password" -AsSecureString; $ptr = [Runtime.InteropServices.Marshal]::SecureStringToBSTR($secure); try { $env:CARGO_APK_RELEASE_KEYSTORE_PASSWORD = [Runtime.InteropServices.Marshal]::PtrToStringBSTR($ptr); cargo apk run --lib --release } finally { [Runtime.InteropServices.Marshal]::ZeroFreeBSTR($ptr); Remove-Item Env:CARGO_APK_RELEASE_KEYSTORE_PASSWORD -ErrorAction SilentlyContinue }
