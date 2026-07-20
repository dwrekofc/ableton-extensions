#!/bin/sh
set -eu

data_dir=${ABLETON_PALETTE_DATA_DIR:-"${HOME}/Library/Application Support/Ableton Command Palette"}
pid_file="$data_dir/daemon.pid"
launch_label="com.personal.ableton-command-palette-daemon"

if [ "$(uname -s)" = "Darwin" ]; then
  if launchctl remove "$launch_label" >/dev/null 2>&1; then
    echo "daemon stopped (launchd job $launch_label)"
  else
    echo "daemon is not registered with launchd"
  fi
  exit 0
fi

if [ ! -f "$pid_file" ]; then
  echo "daemon is not recorded as running"
  exit 0
fi

daemon_pid=$(sed -n '1p' "$pid_file")
if [ -n "$daemon_pid" ] && kill -0 "$daemon_pid" 2>/dev/null; then
  kill "$daemon_pid"
  echo "daemon stopped (pid $daemon_pid)"
else
  echo "removed stale daemon record"
fi
rm -f -- "$pid_file"
