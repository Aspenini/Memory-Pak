#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"
rm -rf Generated
mkdir -p Generated/Assets.xcassets/AppIcon.appiconset
cp ../../icons/ios/*.png Generated/Assets.xcassets/AppIcon.appiconset/
cp ../../icons/ios/Contents.json Generated/Assets.xcassets/AppIcon.appiconset/Contents.json
cat > Generated/Assets.xcassets/Contents.json <<'JSON'
{"info":{"author":"xcode","version":1}}
JSON
xcodegen generate
