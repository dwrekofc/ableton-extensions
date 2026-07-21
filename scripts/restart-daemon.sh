#!/bin/sh
set -eu

project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
"$project_dir/scripts/stop-daemon.sh"
"$project_dir/scripts/start-daemon.sh"
