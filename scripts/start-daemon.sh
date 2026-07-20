#!/bin/sh
set -eu

project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
data_dir=${ABLETON_PALETTE_DATA_DIR:-"${HOME}/Library/Application Support/Ableton Command Palette"}
built_daemon="$project_dir/target/release/ableton-palette-daemon"
daemon="$built_daemon"
cli="$project_dir/target/release/ableton-palette"
pid_file="$data_dir/daemon.pid"
log_file="$data_dir/logs/daemon.log"
launch_label="com.personal.ableton-command-palette-daemon"

if [ ! -x "$built_daemon" ] || [ ! -x "$cli" ]; then
  cargo build --release --workspace --manifest-path "$project_dir/Cargo.toml"
fi
mkdir -p "$data_dir/logs"

if [ "$(uname -s)" = "Darwin" ]; then
  if "$cli" --data-dir "$data_dir" status >/dev/null 2>&1; then
    echo "daemon is already running"
    exit 0
  fi
  launchctl remove "$launch_label" >/dev/null 2>&1 || true
  mkdir -p "$data_dir/bin"
  installed_daemon="$data_dir/bin/ableton-palette-daemon"
  staging_daemon="$installed_daemon.new"
  cp "$built_daemon" "$staging_daemon"
  chmod 700 "$staging_daemon"
  mv "$staging_daemon" "$installed_daemon"
  daemon="$installed_daemon"
  "$daemon" --data-dir "$data_dir" --init >/dev/null
  launchctl submit -l "$launch_label" -o "$log_file" -e "$log_file" -- \
    "$daemon" --data-dir "$data_dir"
  daemon_description="launchd job $launch_label"
else
  "$daemon" --data-dir "$data_dir" --init >/dev/null
  if [ -f "$pid_file" ]; then
    existing_pid=$(sed -n '1p' "$pid_file")
    if [ -n "$existing_pid" ] && kill -0 "$existing_pid" 2>/dev/null; then
      echo "daemon is already running (pid $existing_pid)"
      exit 0
    fi
  fi
  nohup "$daemon" --data-dir "$data_dir" >>"$log_file" 2>&1 &
  daemon_pid=$!
  printf '%s\n' "$daemon_pid" >"$pid_file"
  daemon_description="pid $daemon_pid"
fi

tries=0
while ! "$cli" --data-dir "$data_dir" status >/dev/null 2>&1; do
  tries=$((tries + 1))
  if [ "$tries" -ge 50 ]; then
    echo "daemon did not become ready; inspect $log_file" >&2
    exit 1
  fi
  sleep 0.1
done

echo "daemon started ($daemon_description)"
echo "log: $log_file"
