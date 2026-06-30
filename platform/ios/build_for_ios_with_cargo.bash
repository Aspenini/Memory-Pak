#!/usr/bin/env bash
set -euo pipefail

export PATH="/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin:$PATH:$HOME/.cargo/bin"
export CARGO_TARGET_DIR="$DERIVED_FILE_DIR/cargo"

if [[ "$CONFIGURATION" == "Debug" ]]; then
  profile_args=()
  profile_dir=debug
else
  profile_args=(--release)
  profile_dir=release
  export CARGO_PROFILE_RELEASE_DEBUG="${CARGO_PROFILE_RELEASE_DEBUG:-1}"
fi

case "${PLATFORM_NAME}:${CURRENT_ARCH}" in
  iphoneos:arm64) cargo_target=aarch64-apple-ios ;;
  iphonesimulator:arm64) cargo_target=aarch64-apple-ios-sim ;;
  iphonesimulator:x86_64) cargo_target=x86_64-apple-ios ;;
  *)
    echo "Unsupported iOS build target: ${PLATFORM_NAME}:${CURRENT_ARCH}" >&2
    exit 1
    ;;
esac

(
  cd ../..
  cargo build "${profile_args[@]}" --target "$cargo_target" --bin "$1"
)

source_binary="$CARGO_TARGET_DIR/$cargo_target/$profile_dir/$1"
destination="$TARGET_BUILD_DIR/$EXECUTABLE_PATH"
mkdir -p "$(dirname "$destination")"
cp "$source_binary" "$destination"

if [[ "$CONFIGURATION" != "Debug" ]]; then
  dsymutil "$destination" -o "$DWARF_DSYM_FOLDER_PATH/$DWARF_DSYM_FILE_NAME"
fi
