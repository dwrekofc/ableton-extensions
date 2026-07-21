#!/bin/sh
set -eu

project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
data_dir=${ABLETON_PALETTE_DATA_DIR:-"${HOME}/Library/Application Support/Ableton Command Palette"}
source_app=$("$project_dir/scripts/package-command-bar.sh")
install_root="$data_dir/app"
installed_app="$install_root/Ableton Command Palette.app"

mkdir -p "$install_root"
if [ -d "$installed_app" ]; then
  rm -rf "$installed_app"
fi
cp -R "$source_app" "$installed_app"

echo "installed: $installed_app"
