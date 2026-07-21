#!/bin/sh
set -eu

project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
app_dir="$project_dir/target/release/Ableton Command Palette.app"
contents_dir="$app_dir/Contents"
macos_dir="$contents_dir/MacOS"

if [ "$(uname -s)" != "Darwin" ]; then
  echo "the GPUI command bar currently packages for macOS only" >&2
  exit 1
fi

cargo build --release -p palette-app --manifest-path "$project_dir/Cargo.toml"

if [ -d "$app_dir" ]; then
  rm -rf "$app_dir"
fi
mkdir -p "$macos_dir"
cp "$project_dir/crates/palette-app/macos/Info.plist" "$contents_dir/Info.plist"
cp "$project_dir/target/release/ableton-command-palette" "$macos_dir/ableton-command-palette"
chmod 755 "$macos_dir/ableton-command-palette"

echo "$app_dir"
