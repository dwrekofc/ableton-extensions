# Ableton Command Palette

Keyboard-first access to Ableton Live devices, plug-ins, presets, racks, Max for Live devices, samples, commands, and repeatable workflows through a native GPUI command bar.

## Current state

- Rust daemon, catalog, fuzzy ranking, personalization, workflows, and test CLI.
- Native GPUI command bar invoked with Command-J only while Ableton Live or Ableton Live Beta is frontmost.
- Unified grouped Browser search, scrollable keyboard result navigation, selected-track context, four-position insertion, per-item actions, content filters, execution, and full-catalog refresh.
- Python Remote Script as the primary Live integration for selected context, Browser scanning/loading, four-position placement, and workflow execution.
- Read-only import of Ableton's own plug-in index for instant VST/VST3 names, vendors, types, categories, versions, identifiers, and module paths.
- An optional, experimental TypeScript Ableton extension retained as a future supported-API path; it is not required for the MVP.
- Authenticated local-only protocol and SQLite persistence.
- Automated Rust, Python, TypeScript, and cross-language integration tests.

Start at [manifest.md](manifest.md). To test in Live, follow [docs/ABLETON_SMOKE_TEST.md](docs/ABLETON_SMOKE_TEST.md).

## Start the command bar

```sh
./scripts/start-command-bar.sh
```

Focus Ableton Live or Ableton Live Beta and press `Command-J`. See [docs/COMMAND_BAR.md](docs/COMMAND_BAR.md) for controls and troubleshooting.

## Quick verification

```sh
./scripts/build-all.sh
./scripts/test-integration.sh
./scripts/smoke-test-live-index.sh
```
