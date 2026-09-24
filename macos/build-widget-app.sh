#!/bin/bash
set -euo pipefail

repository="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
app="${1:-$repository/src-tauri/target/release/bundle/macos/Widgitron.app}"
source_dir="$repository/macos/WidgitronWidgets"
extension="$app/Contents/PlugIns/WidgitronWidgets.appex"

if [[ ! -d "$app" ]]; then
  printf 'App bundle missing: %s\nBuild it with pnpm tauri build --bundles app first.\n' "$app" >&2
  exit 1
fi

mkdir -p "$extension/Contents/MacOS"
cp "$source_dir/Info.plist" "$extension/Contents/Info.plist"
swiftc -parse-as-library -O -target arm64-apple-macosx14.0 \
  -framework SwiftUI -framework WidgetKit \
  "$source_dir/WidgitronWidgets.swift" \
  -o "$extension/Contents/MacOS/WidgitronWidgets"

# Local builds use ad-hoc signatures. Distribution builds need the team's
# Developer ID / App Store identity, matching App Group capability, and notarization.
codesign --force --sign - --entitlements "$source_dir/Widget.entitlements" "$extension"
codesign --force --sign - --entitlements "$source_dir/Host.entitlements" "$app"
codesign --verify --deep --strict --verbose=2 "$app"
printf 'Built WidgetKit app: %s\n' "$app"
