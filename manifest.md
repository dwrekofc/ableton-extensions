# Project Manifest

## Start here

This file is the navigation hub and source-of-truth map for the Ableton Command Palette project.

## Product and project documents

- [PRD.md](PRD.md) — intended user outcomes, product scope, architecture decisions, success measures, and roadmap.
- [notices.md](notices.md) — active compatibility, licensing, data, and development notices for builders and users.
- [lessons.md](lessons.md) — chronological record of durable discoveries, constraints, and proven approaches.
- [AGENTS.md](AGENTS.md) — working rules for coding agents and contributors.
- [CLAUDE.md](CLAUDE.md) — symlink to `AGENTS.md` for compatible tooling.
- [README.md](README.md) — concise project entry point and verification commands.
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) — component boundaries, runtime flow, and technology decisions.
- [docs/PROTOCOL.md](docs/PROTOCOL.md) — authenticated local protocol and capability map.
- [docs/ABLETON_SMOKE_TEST.md](docs/ABLETON_SMOKE_TEST.md) — exact install, Live test, log, recovery, and rollback procedure.
- [docs/ROADMAP.md](docs/ROADMAP.md) — completed foundation, current validation gate, GPUI build sequence, and deferred SDK work.

## Repository areas

- `extensions-sdk-1.0.0-beta.0/` — vendor SDK, documentation, examples, and local package archives supplied by Ableton.
- `refs/` — external reference repositories used for research. Reference code is not automatically product code.
- `crates/` — Rust workspace crates for shared models, protocol, persistence, workflows, daemon, and CLI.
- `remote-scripts/` — Python Ableton MIDI Remote Script compatibility bridge.
- `extension/` — optional experimental TypeScript Ableton extension; not required by the primary product path.
- `scripts/` — setup, installation, validation, and smoke-test helpers.
- `docs/` — operator guides, protocol notes, compatibility information, and test procedures.

## Current milestone

The Rust and Python product path has passed real in-Live context, 5,000-item catalog, search, all placement modes, preset/rack loading, workflow, and personalization tests. After restarting Live, recheck the corrected large-catalog transport; then begin the GPUI command bar. The official SDK experiment is sidelined and non-blocking.

## Live validation status

- Rust daemon, CLI, automated tests, and cross-language simulation. **Passed**
- User Library installation and authenticated Live connection. **Passed**
- Selected-track/device context and diagnostics. **Passed**
- Real catalog traversal and native/Browser loading. **Passed at 5,000 items**
- All placement modes, workflows, and personalization. **Passed**
- Corrected transport at 10,000–20,000 items. **Pending Live restart**
- Installation, logs, recovery, and rollback documentation. **Passed**
