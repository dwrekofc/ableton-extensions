#!/bin/sh
set -eu

project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cli="$project_dir/target/release/ableton-palette"

if [ ! -x "$cli" ]; then
  cargo build --release --workspace --manifest-path "$project_dir/Cargo.toml"
fi

if [ -n "${ABLETON_LIVE_PLUGIN_DATABASE:-}" ]; then
  "$cli" import-live-index --plugin-database "$ABLETON_LIVE_PLUGIN_DATABASE"
else
  "$cli" import-live-index
fi
