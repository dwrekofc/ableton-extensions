# Product Roadmap

## Completed foundation

- Product requirements, architecture, compatibility notices, lessons journal, and project manifest.
- Rust protocol, authenticated daemon, SQLite catalog, fuzzy search, usage ranking, aliases, tags, favorites, pins, collections, hotkey assignments, workflows, and CLI.
- Python Remote Script with automatic reconnect, selected context, incremental Browser traversal, path-based loading, all four placement modes, diagnostics, and workflow execution.
- Safe installation, local macOS service lifecycle, unit tests, strict linting, and cross-language integration tests.
- Real Ableton Live 12.4.5b7 verification of context, a 5,000-item catalog, fuzzy search, native insertion, Browser preset/rack loading, all placement modes, workflows, and personalization.
- Read-only import of Ableton's live plug-in index, verified with 60 installed plug-ins across 28 vendors, including 15 instruments and 45 effects.

## Native command bar completed

- GPUI macOS overlay with no Electron or browser runtime.
- Global Command-J invocation gated to foreground Ableton Live and Ableton Live Beta.
- Unified catalog search across devices, plug-ins, presets, racks, Max for Live devices, samples, commands, and workflows.
- Keyboard navigation, selected track/device context, four insertion positions, click support, execution feedback, and automatic dismissal.
- Full Browser refresh streamed in bounded batches so the catalog is not constrained by one large protocol message.
- Local `.app` packaging plus install, start, and stop scripts.
- Grouped 200-result scrolling, persistent content visibility filters, Command-K item actions, and disposable close/focus-loss window lifecycle.
- On-demand Browser resolution for plug-ins seeded from Ableton's internal index.

## Current gate

Install the updated Remote Script, restart Live, then visually verify Command-J invocation, search, placement, execution, dismissal, non-Ableton focus blocking, and the complete batched Browser refresh.

This gate is about catalog scale, not core product behavior. The smaller real catalog and every execution feature already pass.

In parallel, match imported plug-in identifiers to the Plug-Ins Browser root so index-seeded search results become safely loadable without scanning unrelated User Library content.

## Next: product depth

1. Extend the existing in-palette Favorite action with pin, collection, alias, tag, and workflow editing actions.
2. Register dedicated item and workflow hotkeys from stored bindings.
3. Add settings and diagnostics views without weakening the fast overlay interaction.
4. Reconcile internal plug-in-index entries with live Browser paths for guaranteed loading.
5. Measure launch, invocation, query, refresh, and execution latency in production sets.

## Later: optional official extension

The TypeScript extension remains experimental source code only. Revisit it when Ableton's public SDK can expose current selection, Browser discovery, or arbitrary Browser loading well enough to reduce reliance on compatibility behavior. It must not block the GPUI product or duplicate a working path without a clear user benefit.

## Distribution readiness

- Run compatibility checks across supported Live releases.
- Add code signing, notarization, packaging, upgrade, and rollback flows.
- Audit all third-party code and notices before distribution.
- Do not redistribute Ableton's Extensions SDK; builders must obtain it directly under Ableton's license.
