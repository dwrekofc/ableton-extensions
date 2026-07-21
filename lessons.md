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

- An all-track device inventory must recurse through each device's rack chains; selected-track context and top-level `Track.devices` alone cannot find Max for Live devices nested inside Audio Effect Racks. Device names and classes are visible through the Live Object Model, but the source `.amxd` filename/version is not guaranteed to be exposed at runtime.
- The authenticated Python bridge connected to Live Beta 12.4.5b7 and reported selected track, selected device, device order, Live version, and all twelve expected Browser roots with no warnings.
- Incremental scanning and persistence passed with 5,000 real Browser items. Fuzzy search resolved a user EQ Eight preset, and path-based loading successfully inserted that preset at beginning, before selected, after selected, and end.
- Native insertion passed all four placement modes with Live-confirmed device order. Browser loading also succeeded for an `.adg` Audio Effect Rack, proving the route is not limited to stock device names.
- A saved workflow renamed the track and inserted EQ Eight plus Utility in order. Aliases, custom tags, favorites, pins, usage counts, collections, and stored hotkey bindings all round-tripped against the real catalog.
- A 10,000-item scan revealed a truncated JSON frame caused by the bridge's short socket timeout during `sendall`. The correction is installed but requires Live to restart before the in-process Python module changes.
- The official extension package installed successfully, and Ableton's Extension Host showed no crash, but it remained inactive as a daemon peer. Because the Python bridge already provides the complete MVP and more context, the extension is now optional and non-blocking.
- Public source must exclude both research references and Ableton's SDK distribution. Ableton's SDK license permits application development and distribution but expressly prohibits redistributing the SDK outside the application.

## 2026-07-20 — Ableton database import

- Live maintains a small SQLite plug-in catalog at `Live-plugins-1.db` and a much larger versioned file index such as `Live-files-12300.db`. On this machine the plug-in catalog contains the exact identifiers, names, vendors, versions, SDK versions, subcategories, enable/scan state, and module paths needed for fast VST/VST3 discovery.
- A read-only import found 60 enabled and successfully scanned plug-ins from 28 vendors: 15 instruments and 45 audio effects. Search returned real VST and VST3 variants of Serum with the correct track compatibility and module metadata.
- The file index contains more than 115,000 records and several internal relationship/keyword tables. It can accelerate later preset discovery, but its undocumented enums and hierarchy should not be coupled directly to the product model.
- Database discovery and Browser execution are different concerns. An indexed plug-in is marked unresolved until a live Browser item is matched, because a module path or device identifier is not itself a callable Live object.

## 2026-07-20 — Browser sample normalization feasibility

- Ableton Extensions SDK `1.0.0` can add context actions to objects in the Live Set, including `AudioClip`, `Sample`, `Simpler`, and selection scopes for clip slots and Arrangement lanes. It has no context-menu scope or selection argument for Live Browser items, so it cannot receive one or more files selected in the Browser.
- The SDK documentation limits direct filesystem access to an Extension's storage and temporary directories and warns against using child processes or other workarounds to access arbitrary paths. In-place Browser-file normalization is therefore outside both the current API surface and its documented permission model.
- A supported approximation can normalize files referenced by Live Set clips or loaded Simpler samples only if destructive file processing is delegated to a separately installed helper with an explicit user trust and recovery model. That is a materially different workflow and must not be presented as Browser integration.
- Peak normalization should remain preview-first and require an explicit confirmation because an atomic sibling-file replacement is not part of Live's undo history, can invalidate `.asd` analysis, and changes every Live Set that references the same source file.
- A macOS sibling-temp `mv -f` test confirmed that replacement changes the inode, resets the original modification time and mode, and drops the destination's extended attributes. “Same everything except gain” therefore requires an explicit metadata-copy policy and still cannot preserve inode or hard-link identity while retaining atomic replacement.

## 2026-07-20 — Native GPUI command bar

- The current `dwrekofc/zed` GPUI fork is already proven by the user's other native Rust applications and supports runtime Metal shaders, avoiding a full-Xcode shader build dependency. It is a better compatibility base than Loungy's older pinned GPUI API while preserving Loungy's useful window and hotkey patterns.
- `NSWorkspace.frontmostApplication` provides the foreground bundle identity without Accessibility permission. Both installed Live and Live Beta report `com.ableton.live`, so one focus gate covers both; the visible palette remains allowed to consume Command-J again so the shortcut can toggle it closed.
- A hidden GPUI popup can remain resident as a lightweight agent app. `global-hotkey` delivers Command-J independently of Live, while the app decides whether the foreground identity is eligible before activating its window.
- Complete Browser scans cannot safely accumulate into one JSON response: a six-figure catalog can exceed the 32 MiB frame ceiling even when transport timeouts are generous. Streaming bounded catalog batches to the daemon for incremental SQLite upserts removes that message-size ceiling while preserving one final scan summary for the caller.
- Internal plug-in-index discovery and Browser resolution remain separate. An indexed name is discovery metadata, not a load handle; unresolved plug-ins must be reconciled to a live Browser item before execution.

## 2026-07-20 — Command bar interaction smoke test

- `Track.insert_device(name, index)` is not a dependable third-party plug-in loader: Trackspacer 2.5 was present in Live's Plug-Ins Browser but rejected by direct insertion. Database-only plug-ins now resolve exact names inside the Plug-Ins tree and load via `Browser.load_item`.
- A fixed server result limit is also a UI limit. The first palette requested twelve matches and therefore could neither render nor scroll beyond twelve. The app now requests the daemon's 200-item maximum and owns a tracked vertical scroll region whose child index accounts for category headers.
- A hidden agent application and a hidden popup window are different lifecycle states. Destroying the GPUI popup on Escape/focus loss, while keeping only the hotkey agent resident, gives a stronger guarantee than application hiding and prevents stale overlays across applications.
- Category headers must not enter the selection model. Results are grouped only for display/order; keyboard selection remains indexed exclusively over catalog items, with a result-to-rendered-child mapping for scrolling.
- Search visibility is a UI preference, not catalog deletion. Persisting independent group toggles in `ui-settings.json` lets users change the command surface without rescanning or mutating their Ableton-derived catalog.
