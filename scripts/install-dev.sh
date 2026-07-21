#!/bin/sh
set -eu

project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)

"$project_dir/scripts/build-all.sh"
"$project_dir/target/release/ableton-palette-daemon" --init
"$project_dir/scripts/install-remote-script.sh"
"$project_dir/scripts/install-command-bar.sh"

echo "primary installation is staged; use scripts/start-command-bar.sh after enabling the Remote Script in Live"
echo "the optional SDK experiment can be built separately with scripts/build-extension.sh"
