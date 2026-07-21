# Architecture

## Outcome-oriented view

The system separates fast search and personal preferences from Ableton-specific behavior. This keeps the future command bar responsive and allows Live compatibility fixes without rewriting the product.

```text
Future GPUI command bar / current CLI
                 |
       authenticated local protocol
                 |
        Rust daemon and core
       /                     \
Python Remote Script    Optional TS experiment
primary integration     future supported path
       |                     |
       +------ Ableton Live--+
```

## Components

- `palette-protocol` is the shared contract for context, catalog items, requests, results, errors, workflows, collections, and hotkeys.
- `palette-core` stores the catalog and preferences in SQLite, ranks fuzzy results with `nucleo-matcher`, and validates workflows.
- `palette-core` also reads Ableton's `Live-plugins-1.db` in read-only mode to seed enabled, successfully scanned VST/VST3 metadata without a broad Browser crawl.
- `palette-daemon` owns runtime state, authenticates peers, routes requests, records scans and usage, and isolates the eventual GPUI process from Live restarts.
- `palette-cli` exposes every backend capability without a UI so Live integration can be proven first.
- `AbletonCommandPalette` is a Python MIDI Remote Script running on Live's main thread. It discovers selected context, scans and resolves Browser items, loads arbitrary supported Browser content, and executes workflows.
- The optional TypeScript extension uses Ableton's beta Extensions SDK for supported native insertion. It is sidelined because the SDK does not expose the selection or Browser capabilities required by the palette and the primary bridge already covers the complete MVP.

## Key decisions

- Rust is the product core and future UI language; GPUI remains isolated to the deferred frontend crate.
- Loopback TCP plus newline-delimited JSON lets Rust, Python, and TypeScript communicate without platform-specific FFI.
- Every peer must present the locally generated token and exact protocol version.
- Live objects never cross the protocol. Browser paths and stable hashes are used because Live object references are session-bound.
- The Python network worker never touches Live. Requests are drained and executed through the Remote Script's scheduled main-thread callback.
- SQLite holds indexed catalog and personalization data. Runtime configuration and workflow import files remain human-readable JSON.
- Browser compatibility code is isolated because it is not covered by the public Extensions SDK.
- Ableton's internal database schema is undocumented and version-sensitive. Imported plug-ins are marked `browser_resolved: false` until a Live Browser item is matched; the database path alone is never treated as permission to load a plug-in.

## Runtime data

On macOS, runtime state defaults to `~/Library/Application Support/Ableton Command Palette/`:

- `config.json` — loopback address, protocol version, and owner-readable authentication token.
- `palette.sqlite3` — catalog, favorites, pins, usage, aliases, tags, collections, workflows, and hotkey assignments.
- `logs/daemon.log` — daemon output when launched with the provided script.

Set `ABLETON_PALETTE_DATA_DIR` to use an isolated location.

On macOS, `scripts/start-daemon.sh` copies the release daemon into the local application-data directory and registers it as a user-scoped, session-lifetime `launchd` job so Live can reconnect after terminal windows close. `scripts/stop-daemon.sh` removes that exact job. It is not a login item and is not installed system-wide.
