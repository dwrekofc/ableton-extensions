#!/bin/sh
set -eu

project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
data_dir=${ABLETON_PALETTE_DATA_DIR:-"${HOME}/Library/Application Support/Ableton Command Palette"}
cli="$project_dir/target/release/ableton-palette"

if [ ! -x "$cli" ]; then
  echo "release CLI is not built; run scripts/build-all.sh" >&2
  exit 1
fi

"$cli" --data-dir "$data_dir" status
