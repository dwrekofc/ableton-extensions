# Local Integration Protocol

Protocol version: `1`

The daemon binds to `127.0.0.1:49371` by default. Messages are newline-delimited JSON with a 32 MiB frame limit. The generated runtime token is required even though the listener is loopback-only.

## Session

Each connection begins with `hello`, including the protocol version, token, role (`client`, `bridge`, or `extension`), peer name, and capabilities. The daemon replies with `hello_ack` before accepting requests.

Requests and responses carry a unique `request_id`. Errors contain a stable code, user-readable message, and retryable flag. Integration requests time out after 120 seconds.

## Daemon capabilities

- Health and connection status.
- Fuzzy catalog search and usage ranking.
- Favorites and pins.
- Add/remove aliases and custom tags.
- Create/list/delete collections and add/remove members.
- Save/list/delete/run workflows.
- Set/clear/list direct hotkey assignments for the future GPUI app.

## Python bridge capabilities

- Selected track and selected device context.
- Incremental Browser catalog scans.
- Browser item loading at beginning, before selection, after selection, or end.
- Native insertion through Live's compatibility API.
- Multi-action workflows and diagnostics.

## Optional official extension capabilities

- Native device insertion at beginning or end of an explicitly targeted audio or MIDI track.
- Selected-device-relative placement is rejected clearly because the beta public SDK does not expose the selected device.

This peer is experimental and non-blocking. The primary Rust/Python path does not require an extension connection.

The canonical Rust definitions live in `crates/palette-protocol/src/lib.rs`. Python and TypeScript intentionally use small transport adapters rather than generated clients so their Live runtimes stay dependency-light.
