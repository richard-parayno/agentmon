#!/bin/sh
# Wraps target/release/agentmon in target/release/Agentmon.app, so it launches with `open`,
# from Finder or as a login item. LSUIElement keeps it out of the Dock (the app also asks for
# the accessory policy itself, so `cargo run` behaves the same).
set -eu

cd "$(dirname "$0")/.."
version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
app=target/release/Agentmon.app

rm -rf "$app"
mkdir -p "$app/Contents/MacOS"
cp target/release/agentmon "$app/Contents/MacOS/agentmon"
cat > "$app/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>Agentmon</string>
  <key>CFBundleIdentifier</key><string>com.richardparayno.agentmon</string>
  <key>CFBundleExecutable</key><string>agentmon</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>$version</string>
  <key>CFBundleVersion</key><string>$version</string>
  <key>LSMinimumSystemVersion</key><string>13.0</string>
  <key>LSUIElement</key><true/>
  <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
EOF
# Ad-hoc signature, so macOS runs the bundle as one app.
codesign --force --sign - "$app" >/dev/null 2>&1 || true
echo "$app"
