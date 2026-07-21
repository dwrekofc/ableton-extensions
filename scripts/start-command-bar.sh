#!/bin/sh
set -eu

project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
data_dir=${ABLETON_PALETTE_DATA_DIR:-"${HOME}/Library/Application Support/Ableton Command Palette"}
installed_app="$data_dir/app/Ableton Command Palette.app"

cd "$project_dir"
cargo build --release -p palette-daemon -p palette-cli
"$project_dir/scripts/restart-daemon.sh"
"$project_dir/scripts/install-command-bar.sh"
pkill -x ableton-command-palette >/dev/null 2>&1 || true
open -n "$installed_app"

echo "command bar is running; focus Ableton Live or Ableton Live Beta and press Command-J"
