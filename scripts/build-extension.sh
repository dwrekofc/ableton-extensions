#!/bin/sh
set -eu

project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
sdk_archive="$project_dir/extensions-sdk-1.0.0-beta.0/ableton-extensions-sdk-1.0.0-beta.0.tgz"

if [ ! -f "$sdk_archive" ]; then
  echo "Ableton Extensions SDK 1.0.0-beta.0 is required locally and is not redistributed by this repository." >&2
  exit 1
fi

cd "$project_dir/extension"
npm ci
npm test
npm run package
