# Local Integration Protocol

Protocol version: `1`

The daemon binds to `127.0.0.1:49371` by default. Messages are newline-delimited JSON with a 32 MiB frame limit. The generated runtime token is required even though the listener is loopback-only.

## Session

Each connection begins with `hello`, including the protocol version, token, role (`client`, `bridge`, or `extension`), peer name, and capabilities. The daemon replies with `hello_ack` before accepting requests.

Requests and responses carry a unique `request_id`. Errors contain a stable code, user-readable message, and retryable flag. Integration requests time out after 120 seconds.

Complete Browser scans use `catalog_batch` events associated with the pending scan request. The daemon validates that association and upserts each bounded batch immediately. The bridge ends the request with `catalog_scan { scanned }`, keeping every frame below the limit while preserving one completion result for CLI and GPUI clients. The older single-response `catalog` result remains accepted for compatibility.

## Daemon capabilities

- Health and connection status.
- Fuzzy catalog search and usage ranking.
- Read-only import of Ableton's local plug-in database with an explicit import summary.
- Favorites and pins.
- Add/remove aliases and custom tags.
- Create/list/delete collections and add/remove members.
- Save/list/delete/run workflows.
- Set/clear/list direct hotkey assignments for the future GPUI app.

## Python bridge capabilities

- Selected track and selected device context.
- Read-only, case-insensitive device inventory across normal, return, and main tracks, including devices nested in rack chains.
- Incremental Browser catalog scans.
- Browser item loading at beginning, before selection, after selection, or end.
- Exact-name Browser resolution and loading for plug-ins first discovered through Ableton's internal index.
- Native insertion through Live's compatibility API.
- Multi-action workflows and diagnostics.

## Optional official extension capabilities

- Native device insertion at beginning or end of an explicitly targeted audio or MIDI track.
- Selected-device-relative placement is rejected clearly because the beta public SDK does not expose the selected device.

This peer is experimental and non-blocking. The primary Rust/Python path does not require an extension connection.

## Ableton index import

`import_live_database` accepts an optional plug-in database path. When omitted, the daemon discovers `Live-plugins-1.db` in Ableton's standard application-data directory. Only records with `enabled=1` and `scanstate=1` are copied. Imported items carry `source: live_database` and preserve device identifier, vendor, version, SDK version, subtype, format, and module path as metadata.

Importing and Browser discovery remain separate. When an imported item has no path, the daemon requests exact-name resolution in Live's Plug-Ins Browser at execution time; successful resolution is cached for that Live session.

The canonical Rust definitions live in `crates/palette-protocol/src/lib.rs`. Python and TypeScript intentionally use small transport adapters rather than generated clients so their Live runtimes stay dependency-light.
