# Project Lessons

This is a chronological engineering journal. Add an entry whenever the project reveals a durable constraint, successful pattern, failed assumption, or compatibility detail.

## 2026-07-20 — Repository baseline

- The workspace initially contained only the Ableton Extensions SDK distribution and reference repositories; there was no existing product application or Git history at the workspace root.
- The installed compatible development build is Ableton Live 12.4.5b7.

## 2026-07-20 — Official SDK boundary

- Extensions run as Node.js processes alongside Live and can use npm and Node APIs.
- The SDK can display modal Webviews, persist local data, manipulate tracks and devices, and insert built-in Live devices by name and index.
- The SDK cannot register global shortcuts, expose the selected track/device, enumerate Live's Browser, or load third-party plug-ins, arbitrary presets, racks, or Max for Live devices.
- The SDK modal is result-oriented: the Webview returns a string when it closes. It is not a general continuous frontend/backend bridge.

## 2026-07-20 — Live integration path

- The public Live Object Model exposes selected track, selected device, device insertion mode, and device movement helpers.
- Live's bundled Push Remote Scripts demonstrate internal Browser item traversal and `load_item` behavior. This is the practical path to complete Browser loading, but it must be treated as a version-sensitive compatibility layer.
- Stable Browser identifiers should be derived from paths and fingerprints rather than persisting session-specific Live object IDs.

## 2026-07-20 — UI references and framework choice

- Clui CC creates a polished floating overlay with a transparent, frameless, always-on-top Electron window and renders the visible pill in React.
- Its lightweight appearance does not imply a lightweight runtime; most of the effect comes from transparent click-through regions, careful window behavior, design tokens, and animation.
- GPUI is chosen for the final command bar despite its smaller ecosystem and pre-1.0 status. Shared functionality must therefore remain UI-independent and testable from a CLI.

## 2026-07-20 — Loungy reference review

- Loungy is MIT-licensed and demonstrates a useful Rust launcher split between command modeling, persistence, fuzzy matching, global hotkeys, IPC, platform code, and GPUI rendering.
- `nucleo` is a good fit for native fuzzy matching and can stay independent of GPUI.
- Loungy's single-instance Unix socket is concise but is macOS/Linux-specific, unauthenticated, and reads unframed JSON into fixed buffers. This project needs loopback TCP, newline framing, authentication, request IDs, explicit protocol versions, and size limits so Rust, Python, and TypeScript can interoperate safely.
- Loungy's `global-hotkey` approach can be reused later for the GPUI application, but hotkey registration is outside the current headless milestone.
- Loungy's persistence layer is more capable than the initial palette needs. SQLite is preferable here because catalog, aliases, collections, usage, and workflows benefit from transparent schemas and direct diagnostics.

## 2026-07-20 — Headless integration implementation

- A single authenticated newline-delimited JSON contract works across Rust, Live's dependency-free Python environment, and the Ableton Node.js Extension Host. Simulated bridge and extension peers complete context, scan, search, load, placement, diagnostics, personalization, and workflow flows end to end.
- Live API calls must stay on Live's scheduled main thread. A background Python socket worker can reconnect and queue requests safely as long as it never retains or calls Live objects.
- Catalog scans must be incremental. Processing a bounded number of Browser nodes per scheduled callback avoids monopolizing Live's UI thread.
- The official extension needs an explicit track-target context action because the beta SDK cannot read Live's selected track. It can insert at deterministic chain boundaries, while selected-device-relative placement remains a compatibility-bridge capability.
- Saved workflows should also become catalog items, allowing the same fuzzy search, pins, favorites, tags, aliases, collections, and usage model to apply to actions as well as devices.
- Local runtime authentication is useful even on loopback. The generated token is stored with owner-only permissions on Unix, and all peers reject non-loopback configuration.
- The official extension package builds successfully as `.ablx`; Ableton's documented install path is dropping that package onto the Extensions page in Live Settings.
- The machine currently has Live Suite 12.4.3 running and Live Beta 12.4.5b7 installed. The Remote Script can be smoke-tested in Suite after restart; the beta application remains the correct target for the `1.0.0-beta.0` official extension host.
- A plain detached child is not a reliable lifecycle model in every build or terminal environment. The macOS test launcher now uses a user-scoped `launchd` job, while the future GPUI app will own the daemon lifecycle directly.
- Launching a freshly rebuilt service executable directly from the external project volume can stall in macOS's dynamic loader under `launchd`. Installing the daemon binary into local application data before submission avoids that volume-lifecycle dependency.
- Reusing the bridge's 200 ms receive-poll timeout for multi-megabyte catalog writes can time out `sendall` mid-frame. Large loopback writes need a separate generous timeout; otherwise Rust receives truncated JSON and the bridge reconnects even though smaller scans succeed.

## 2026-07-20 — Real Ableton smoke test

- The authenticated Python bridge connected to Live Beta 12.4.5b7 and reported selected track, selected device, device order, Live version, and all twelve expected Browser roots with no warnings.
- Incremental scanning and persistence passed with 5,000 real Browser items. Fuzzy search resolved a user EQ Eight preset, and path-based loading successfully inserted that preset at beginning, before selected, after selected, and end.
- Native insertion passed all four placement modes with Live-confirmed device order. Browser loading also succeeded for an `.adg` Audio Effect Rack, proving the route is not limited to stock device names.
- A saved workflow renamed the track and inserted EQ Eight plus Utility in order. Aliases, custom tags, favorites, pins, usage counts, collections, and stored hotkey bindings all round-tripped against the real catalog.
- A 10,000-item scan revealed a truncated JSON frame caused by the bridge's short socket timeout during `sendall`. The correction is installed but requires Live to restart before the in-process Python module changes.
- The official extension package installed successfully, and Ableton's Extension Host showed no crash, but it remained inactive as a daemon peer. Because the Python bridge already provides the complete MVP and more context, the extension is now optional and non-blocking.
- Public source must exclude both research references and Ableton's SDK distribution. Ableton's SDK license permits application development and distribution but expressly prohibits redistributing the SDK outside the application.
