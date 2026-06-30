# Android build

Install Android SDK 35, the NDK, `cargo-ndk`, and the Rust
`aarch64-linux-android` and `x86_64-linux-android` targets. Then run:

```sh
./gradlew :app:assembleDebug
./gradlew :app:assembleRelease
```

The package identifier remains `com.Aspenini.MemoryPak`. Slint 1.17 requires
API 26 even though the previous shell targeted API 24.
