#!/bin/sh
set -eu

project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)

cd "$project_dir"
cargo build --release --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
python3 -m unittest discover -s remote-scripts/tests -v
"$project_dir/scripts/test-integration.sh"

echo "primary Rust and Python backend, integration, and tests are ready"
