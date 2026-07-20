# Product Roadmap

## Completed foundation

- Product requirements, architecture, compatibility notices, lessons journal, and project manifest.
- Rust protocol, authenticated daemon, SQLite catalog, fuzzy search, usage ranking, aliases, tags, favorites, pins, collections, hotkey assignments, workflows, and CLI.
- Python Remote Script with automatic reconnect, selected context, incremental Browser traversal, path-based loading, all four placement modes, diagnostics, and workflow execution.
- Safe installation, local macOS service lifecycle, unit tests, strict linting, and cross-language integration tests.
- Real Ableton Live 12.4.5b7 verification of context, a 5,000-item catalog, fuzzy search, native insertion, Browser preset/rack loading, all placement modes, workflows, and personalization.

## Current gate

Restart Live so the corrected Remote Script is loaded, then verify a 10,000–20,000-item catalog scan. The first large scan revealed that the bridge's short receive timeout could interrupt a multi-megabyte write; the write path now has an independent 30-second window.

This gate is about catalog scale, not core product behavior. The smaller real catalog and every execution feature already pass.

## Next: GPUI command palette

1. Add the GPUI application shell and make it own daemon lifecycle.
2. Register the global palette shortcut and render a fast floating overlay.
3. Connect search results to the proven Rust catalog and ranking APIs.
4. Show selected track/device context and placement choice before execution.
5. Add keyboard-only result navigation, favorites, pins, collections, aliases, tags, and workflow actions.
6. Add direct assigned-hotkey registration using the stored bindings.
7. Add settings, diagnostics, re-scan controls, and clear compatibility feedback.
8. Measure launch, invocation, query, and execution latency in normal production sets.

## Later: optional official extension

The TypeScript extension remains experimental source code only. Revisit it when Ableton's public SDK can expose current selection, Browser discovery, or arbitrary Browser loading well enough to reduce reliance on compatibility behavior. It must not block the GPUI product or duplicate a working path without a clear user benefit.

## Distribution readiness

- Run compatibility checks across supported Live releases.
- Add code signing, notarization, packaging, upgrade, and rollback flows.
- Audit all third-party code and notices before distribution.
- Do not redistribute Ableton's Extensions SDK; builders must obtain it directly under Ableton's license.
