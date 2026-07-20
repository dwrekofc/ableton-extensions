#!/bin/sh
set -eu

project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
user_library=${ABLETON_USER_LIBRARY:-"${HOME}/Music/Ableton/User Library"}
remote_parent="$user_library/Remote Scripts"
target="$remote_parent/AbletonCommandPalette"
source_dir="$project_dir/remote-scripts/AbletonCommandPalette"

mkdir -p "$remote_parent"
staging=$(mktemp -d "$remote_parent/.ableton-command-palette.XXXXXX")
for source_file in "$source_dir"/*.py; do
  cp "$source_file" "$staging/"
done

if [ -e "$target" ]; then
  timestamp=$(date +%Y%m%d-%H%M%S)
  backup="$target.backup-$timestamp"
  mv "$target" "$backup"
  echo "previous Remote Script moved to $backup"
fi
mv "$staging" "$target"
echo "installed Remote Script at $target"
