# Project Manifest

## Start here

This file is the navigation hub and source-of-truth map for the Ableton Command Palette project.

## Product and project documents

- [PRD.md](PRD.md) — intended user outcomes, product scope, architecture decisions, success measures, and roadmap.
- [BUGS.md](BUGS.md) — reported defects, interaction gaps, status, and acceptance criteria.
- [notices.md](notices.md) — active compatibility, licensing, data, and development notices for builders and users.
- [lessons.md](lessons.md) — chronological record of durable discoveries, constraints, and proven approaches.
- [AGENTS.md](AGENTS.md) — working rules for coding agents and contributors.
- [CLAUDE.md](CLAUDE.md) — symlink to `AGENTS.md` for compatible tooling.
- [README.md](README.md) — concise project entry point and verification commands.
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) — component boundaries, runtime flow, and technology decisions.
- [docs/PROTOCOL.md](docs/PROTOCOL.md) — authenticated local protocol and capability map.
- [docs/ABLETON_SMOKE_TEST.md](docs/ABLETON_SMOKE_TEST.md) — exact install, Live test, log, recovery, and rollback procedure.
- [docs/COMMAND_BAR.md](docs/COMMAND_BAR.md) — command-bar startup, controls, execution behavior, refresh, and troubleshooting.
- [docs/ROADMAP.md](docs/ROADMAP.md) — completed foundation, current validation gate, GPUI build sequence, and deferred SDK work.

## Repository areas

- `extensions-sdk-1.0.0-beta.0/` — vendor SDK, documentation, examples, and local package archives supplied by Ableton.
- `refs/` — external reference repositories used for research. Reference code is not automatically product code.
- `crates/` — Rust workspace crates for shared models, protocol, persistence, workflows, daemon, CLI, and the native GPUI app.
- `remote-scripts/` — Python Ableton MIDI Remote Script compatibility bridge.
- `extension/` — optional experimental TypeScript Ableton extension; not required by the primary product path.
- `scripts/` — setup, installation, validation, and smoke-test helpers.
- `docs/` — operator guides, protocol notes, compatibility information, and test procedures.

## Current milestone

The native GPUI command bar is implemented and packaged with an Ableton-only Command-J gate, unified search, keyboard navigation, selected-track context, placement, execution, and batched full-Browser refresh. The next gate is an in-Live visual and shortcut smoke test plus a full refresh after reloading the updated Remote Script. The official SDK experiment remains sidelined and non-blocking.

## Live validation status

- Rust daemon, CLI, automated tests, and cross-language simulation. **Passed**
- User Library installation and authenticated Live connection. **Passed**
- Selected-track/device context and diagnostics. **Passed**
- Real catalog traversal and native/Browser loading. **Passed at 5,000 items**
- Read-only Ableton plug-in index import and search. **Passed: 60 plug-ins, 28 vendors**
- All placement modes, workflows, and personalization. **Passed**
- Corrected transport at 10,000–20,000 items. **Pending Live restart**
- GPUI app build, unit tests, packaging, and static integration checks. **Passed**
- Command-J invocation and visual behavior in Live. **Pending live UI smoke test**
- Installation, logs, recovery, and rollback documentation. **Passed**
