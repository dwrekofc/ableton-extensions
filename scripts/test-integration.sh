#!/bin/sh
set -eu

project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
test_dir=$(mktemp -d /tmp/ableton-palette-test.XXXXXX)
daemon_pid=""
bridge_pid=""
extension_pid=""

cleanup() {
  [ -z "$extension_pid" ] || kill "$extension_pid" 2>/dev/null || true
  [ -z "$bridge_pid" ] || kill "$bridge_pid" 2>/dev/null || true
  [ -z "$daemon_pid" ] || kill "$daemon_pid" 2>/dev/null || true
  case "$test_dir" in
    /tmp/ableton-palette-test.*) rm -rf -- "$test_dir" ;;
  esac
}
trap cleanup EXIT INT TERM

port=$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1]); s.close()')

cd "$project_dir"
cargo build --workspace >/dev/null
target/debug/ableton-palette-daemon --data-dir "$test_dir" --port "$port" >"$test_dir/daemon.log" 2>&1 &
daemon_pid=$!

tries=0
while [ ! -f "$test_dir/config.json" ]; do
  tries=$((tries + 1))
  [ "$tries" -lt 100 ] || { echo "daemon configuration timeout" >&2; exit 1; }
  sleep 0.05
done

python3 scripts/fake-live-peer.py --data-dir "$test_dir" --role bridge &
bridge_pid=$!
python3 scripts/fake-live-peer.py --data-dir "$test_dir" --role extension &
extension_pid=$!

tries=0
while :; do
  status=$(target/debug/ableton-palette --data-dir "$test_dir" status 2>/dev/null || true)
  echo "$status" | grep -q '"bridge_connected": true' && echo "$status" | grep -q '"extension_connected": true' && break
  tries=$((tries + 1))
  [ "$tries" -lt 100 ] || { echo "integration peers did not connect" >&2; exit 1; }
  sleep 0.05
done

target/debug/ableton-palette --data-dir "$test_dir" context | grep -q '"track_name": "Audio 1"'
target/debug/ableton-palette --data-dir "$test_dir" inspect-devices "Swiss Army" | grep -q '"device_name": "CTZ Swiss Army Meter"'
target/debug/ableton-palette --data-dir "$test_dir" scan --max-items 100 --max-depth 4 | grep -q '"scanned": 1'
target/debug/ableton-palette --data-dir "$test_dir" search verb | grep -q '"id": "browser:fake-reverb"'
target/debug/ableton-palette --data-dir "$test_dir" alias browser:fake-reverb spaceverb | grep -q '"kind": "ack"'
target/debug/ableton-palette --data-dir "$test_dir" tag browser:fake-reverb spacious | grep -q '"kind": "ack"'
target/debug/ableton-palette --data-dir "$test_dir" search spacious | grep -q '"id": "browser:fake-reverb"'
target/debug/ableton-palette --data-dir "$test_dir" favorite browser:fake-reverb | grep -q '"kind": "ack"'
target/debug/ableton-palette --data-dir "$test_dir" pin browser:fake-reverb | grep -q '"kind": "ack"'
target/debug/ableton-palette --data-dir "$test_dir" create-collection Testing | grep -q '"kind": "ack"'
target/debug/ableton-palette --data-dir "$test_dir" add-to-collection Testing browser:fake-reverb | grep -q '"kind": "ack"'
target/debug/ableton-palette --data-dir "$test_dir" list-collections | grep -q '"name": "Testing"'
target/debug/ableton-palette --data-dir "$test_dir" set-hotkey cmd+shift+r browser:fake-reverb | grep -q '"kind": "ack"'
target/debug/ableton-palette --data-dir "$test_dir" list-hotkeys | grep -q '"accelerator": "cmd+shift+r"'
target/debug/ableton-palette --data-dir "$test_dir" load browser:fake-reverb --position after | grep -q '"kind": "ack"'
target/debug/ableton-palette --data-dir "$test_dir" insert-native Reverb --position end | grep -q '"kind": "ack"'
target/debug/ableton-palette --data-dir "$test_dir" sdk-insert-native Reverb --position end | grep -q '"kind": "ack"'
target/debug/ableton-palette --data-dir "$test_dir" diagnostics | grep -q '"component": "fake_bridge"'

cat >"$test_dir/workflow.json" <<'JSON'
{
  "id": "test-chain",
  "name": "Test Chain",
  "description": "Cross-language protocol test",
  "actions": [
    {
      "type": "load_item",
      "item_id": "browser:fake-reverb",
      "position": "end"
    },
    {
      "type": "rename_track",
      "name": "Protocol Test Track"
    }
  ]
}
JSON

target/debug/ableton-palette --data-dir "$test_dir" save-workflow "$test_dir/workflow.json" | grep -q '"id": "test-chain"'
target/debug/ableton-palette --data-dir "$test_dir" list-workflows | grep -q '"id": "test-chain"'
target/debug/ableton-palette --data-dir "$test_dir" search "Test Chain" | grep -q '"kind": "workflow"'
target/debug/ableton-palette --data-dir "$test_dir" run-workflow test-chain | grep -q '"action_type": "rename_track"'

echo "cross-language integration test passed"
