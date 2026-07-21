#!/bin/sh
set -eu

if pkill -x ableton-command-palette >/dev/null 2>&1; then
  echo "command bar stopped"
else
  echo "command bar was not running"
fi
