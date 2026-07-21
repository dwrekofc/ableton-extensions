#!/bin/sh
set -eu

project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cli="$project_dir/target/release/ableton-palette"
plugin_database=${ABLETON_LIVE_PLUGIN_DATABASE:-"${HOME}/Library/Application Support/Ableton/Live Database/Live-plugins-1.db"}
test_dir=$(mktemp -d /tmp/ableton-live-index-smoke.XXXXXX)

cleanup() {
  case "$test_dir" in
    /tmp/ableton-live-index-smoke.*) rm -rf -- "$test_dir" ;;
  esac
}
trap cleanup EXIT INT TERM

[ -f "$plugin_database" ] || { echo "Ableton plug-in index not found: $plugin_database" >&2; exit 1; }
command -v sqlite3 >/dev/null 2>&1 || { echo "sqlite3 is required for the smoke test" >&2; exit 1; }

"$cli" --json import-live-index --plugin-database "$plugin_database" >"$test_dir/import.json"
python3 - "$test_dir/import.json" <<'PY'
import json
import sys

with open(sys.argv[1], "r", encoding="utf-8") as handle:
    summary = json.load(handle)["result"]["data"]
assert summary["imported"] > 0, "no plug-ins imported"
assert summary["instruments"] > 0, "no instruments imported"
assert summary["audio_effects"] > 0, "no audio effects imported"
print(
    "imported {imported} plug-ins: {instruments} instruments, "
    "{audio_effects} effects, {vendors} vendors".format(**summary)
)
PY

sample_name=$(sqlite3 -readonly "$plugin_database" \
  "SELECT name FROM plugins WHERE enabled=1 AND scanstate=1 ORDER BY name COLLATE NOCASE LIMIT 1")
[ -n "$sample_name" ] || { echo "Ableton plug-in index has no enabled plug-ins" >&2; exit 1; }
"$cli" --json search "$sample_name" --limit 20 >"$test_dir/search.json"
python3 - "$test_dir/search.json" "$sample_name" <<'PY'
import json
import sys

with open(sys.argv[1], "r", encoding="utf-8") as handle:
    items = json.load(handle)["result"]["data"]
name = sys.argv[2]
matches = [item for item in items if item["name"] == name and item["source"] == "live_database"]
assert matches, "imported plug-in was not returned by search: " + name
item = matches[0]
assert item["metadata"]["device_identifier"].startswith("device:")
assert item["metadata"]["module_path"]
print("search verified: %s (%s)" % (item["name"], item["metadata"]["plugin_format"]))
PY

echo "Ableton Live index smoke test passed"
