# iOS build

Install Xcode, Xcodegen, and the Rust `aarch64-apple-ios`,
`aarch64-apple-ios-sim`, and `x86_64-apple-ios` targets. Generate the project:

```sh
./generate.bash
xcodebuild -project MemoryPak.xcodeproj -scheme MemoryPak \
  -sdk iphonesimulator -configuration Debug build
```

The deployment target is iOS 12 and the bundle identifier remains
`com.Aspenini.MemoryPak`.
