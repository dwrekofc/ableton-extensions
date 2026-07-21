# Ableton Command Palette

Keyboard-first access to Ableton Live devices, plug-ins, presets, racks, Max for Live devices, commands, and repeatable workflows. This repository currently contains the complete headless backend and Live integrations; the GPUI command-bar frontend is the next milestone.

## Current state

- Rust daemon, catalog, fuzzy ranking, personalization, workflows, and test CLI.
- Python Remote Script as the primary Live integration for selected context, Browser scanning/loading, four-position placement, and workflow execution.
- Read-only import of Ableton's own plug-in index for instant VST/VST3 names, vendors, types, categories, versions, identifiers, and module paths.
- An optional, experimental TypeScript Ableton extension retained as a future supported-API path; it is not required for the MVP.
- Authenticated local-only protocol and SQLite persistence.
- Automated Rust, Python, TypeScript, and cross-language integration tests.

Start at [manifest.md](manifest.md). To test in Live, follow [docs/ABLETON_SMOKE_TEST.md](docs/ABLETON_SMOKE_TEST.md).

## Quick verification

```sh
./scripts/build-all.sh
./scripts/test-integration.sh
./scripts/smoke-test-live-index.sh
```

No GPUI frontend has been added yet by design. The next product milestone is the GPUI command bar after the corrected large-catalog scan is rechecked following a Live restart.
