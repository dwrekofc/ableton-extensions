# Ableton Command Palette — Product Requirements

## Product vision

Ableton Command Palette gives producers immediate, keyboard-first access to the tools and workflows they use inside Ableton Live. It replaces repetitive Browser navigation with one searchable command surface that can load devices and presets, run production commands, and execute repeatable workflows without breaking creative focus.

## Primary outcome

A producer can invoke the palette, describe or search for what they need, choose a result, and have Ableton perform the action at the intended location in the selected track with minimal delay and no mouse navigation.

## User benefits

- Find native devices, plug-ins, presets, racks, and Max for Live devices from one place.
- Add the chosen item at the beginning, before or after the selected device, or at the end of a device chain.
- Turn recurring production sequences into dependable one-step workflows.
- Personalize access through aliases, favorites, pins, collections, usage-based ranking, and dedicated hotkeys.
- Stay focused on music instead of remembering folder locations and navigating nested Browser categories.
- Keep personal configuration and usage data local to the computer.

## Product capabilities

### Universal catalog

The product presents a unified catalog of loadable Ableton content and executable actions. Results can be discovered by name, alias, category, tag, collection, or common shorthand.

Results are grouped by recognizable content type so producers can scan them quickly. Users can hide categories they do not want in search, and those choices persist between sessions.

Ableton's local plug-in index may seed VST/VST3 discovery without crawling unrelated Browser content. The imported index remains read-only and is reconciled with Live Browser items before execution.

### Context-aware execution

Before executing an action, the product understands the selected track, selected device, and requested insertion position. Incompatible results are hidden or clearly explained rather than failing silently.

### Fast personalization

Users can favorite or pin results, create custom collections, add aliases and tags, and assign direct shortcuts. Frequently and recently used items rise naturally in search results.

An item-action surface keeps personalization keyboard-first, beginning with adding or removing the selected result from Favorites.

### Commands and workflows

The product can run individual Ableton actions or validated multi-step workflows such as creating a track, building an effect chain, renaming or arming the track, and applying standard device settings.

### Resilient compatibility

Ableton integration that depends on version-sensitive behavior is isolated from the rest of the product. A Live update should require a small compatibility update rather than a redesign of the catalog, workflows, or future command-bar interface.

## Experience principles

- Keyboard first: every primary action is available without a mouse.
- Immediate: invocation and search should feel instantaneous.
- Predictable: the target track and insertion position are visible before execution.
- Reversible: actions should respect Live's undo behavior wherever the available APIs permit it.
- Local first: no cloud account or hosted service is required.
- Fail safely: unavailable content, incompatible tracks, and stale catalog entries produce actionable feedback.
- Stay out of the way: the palette disappears completely on Escape or when Ableton loses focus and never follows the user into another application.

## High-level architecture and technology decisions

### Native application and command bar

The desktop application is written in Rust and uses GPUI for a small, fast, GPU-rendered native overlay. It remains resident without a Dock icon, registers Command-J globally, and activates only when Ableton Live or Ableton Live Beta is the foreground application.

### Shared Rust core

Reusable Rust crates own the product model, protocol, persistence, ranking, workflow validation, execution requests, and compatibility reporting. These capabilities remain independent of GPUI so they can be tested from a command-line harness first.

### Primary Ableton bridge

A Python MIDI Remote Script runs inside Ableton Live and is the primary MVP integration. It reports Live's current context, catalogs and loads Browser content, performs all four placement modes, and executes workflows. The bridge communicates only over the local machine using a versioned, authenticated protocol.

### Optional official Ableton extension

A TypeScript extension built with the official Ableton Extensions SDK is retained as an experimental, forward-compatible path for supported native operations. It is not required for the MVP and is sidelined until the public SDK exposes enough selection and Browser functionality to provide meaningful benefit over the working Python bridge.

### Local persistence

Catalog metadata, aliases, favorites, collections, hotkeys, workflows, compatibility data, and usage history are stored locally. Human-readable configuration is preferred where practical, with a structured local database available for indexed data.

Ableton-owned databases are never modified or redistributed. A compatibility importer reads only the minimum required schema and copies normalized metadata into the product database.

## Success measures

- The integration can identify the selected Live track and selected device.
- Native devices can be inserted at every supported position.
- Browser items can be cataloged and resolved without blocking normal Live use.
- Plug-ins, presets, racks, and Max for Live devices can be loaded through the compatibility bridge where Live permits it.
- Commands and workflows return clear success or failure results.
- The integration can recover cleanly after Live restarts or the desktop process disconnects.
- Automated tests cover protocol compatibility, placement decisions, workflow validation, persistence, and error handling.
- A CLI test harness can exercise the complete backend independently of the GPUI interface.
- Command-J opens only from Live, search and navigation remain keyboard-first, and the visible target and placement agree with Live before execution.
- Large result sets remain scrollable, category filters persist, and every result is presented under a clear content-group heading.

## Scope for the current build

Build everything required to use and test the product in Ableton Live:

- Rust workspace, shared models, protocol, persistence, catalog, ranking, workflows, daemon, and CLI harness.
- Python Remote Script bridge and installation tooling.
- Optional TypeScript Ableton extension source and separate packaging/build tooling for authorized SDK users.
- Local authentication and lifecycle handling.
- Automated tests and an Ableton smoke-test guide.
- Compatibility diagnostics and structured logging.
- Native GPUI command bar, Ableton-only global shortcut, unified search, context, placement, execution, and full Browser refresh.

## Deferred scope

- Public distribution, code signing, notarization, automatic updates, and commercial licensing work.
- Cloud synchronization or accounts.
- Real-time audio or MIDI processing.

## High-level implementation roadmap

1. Establish the project contract, shared terminology, notices, and durable lessons log.
2. Define and test the versioned local protocol and shared product model.
3. Build the Ableton Remote Script bridge with context, catalog, loading, placement, and diagnostics.
4. Build the Rust daemon, persistent catalog, personalization, ranking, workflow engine, and CLI test harness.
5. Validate the Python bridge in Live, including context, placement, Browser content, workflows, and personalization.
6. Stream large catalogs in bounded batches and recheck a complete refresh after restarting Live.
7. Build and validate the GPUI command-bar frontend on top of the verified Rust and Python backend.
8. Add deeper in-palette personalization and dedicated item/workflow hotkeys.
9. Revisit the optional official extension when its SDK coverage materially improves the product.
