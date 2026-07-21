# Project Notices

This file records important information for builders and users. Keep it current when assumptions, compatibility, installation, or risk changes.

## Current status

- The project is in active development and is not ready for general use.
- The native GPUI command bar is implemented and packaged; its first real Live UI smoke test is pending.
- The primary Rust/Python path passed real Live tests for context, 5,000 catalog items, search, all placement modes, presets, racks, workflows, and personalization.
- A Live restart is required after installing the updated Remote Script before validating batched full-Browser refresh.
- Ableton's plug-in index importer passed against schema version 1 with 60 enabled plug-ins. Imported metadata is searchable immediately; execution now attempts exact-name resolution against the live Plug-Ins Browser and reports a clear error if no loadable match exists.
- The Python bridge is verified in Ableton Live Beta 12.4.5b7. Other Live versions remain unverified until exercised.
- The official Extensions SDK `1.0.0-beta.0` experiment is sidelined and is not required by the primary product path.

## Compatibility notice

- The official Ableton Extensions SDK supports native Live device insertion but does not currently expose global shortcuts, selected track/device state, the Live Browser catalog, or arbitrary plug-in/preset loading.
- The Extensions SDK `1.0.0` context-menu scopes cover objects in the Live Set (including clips, tracks, Simpler, and loaded samples), but do not cover Live Browser items or Browser selections. A Browser right-click action cannot currently be implemented through the supported SDK.
- Extensions are restricted to their storage and temporary directories for direct filesystem access. Destructively rewriting an arbitrary sample in place from an Extension would violate the documented permission model and may stop working when Ableton tightens sandbox enforcement.
- The current ffmpeg reference recipe preserves the encoded audio format when its codec, sample rate, and sample format are pinned, but its sibling-temp rename changes the inode and does not automatically preserve the original file's extended attributes, Finder tags, timestamps, ACLs, or hard-link identity. A production normalizer needs explicit metadata preservation, backup/recovery, and post-write verification.
- Full Browser loading therefore requires a Python Remote Script compatibility bridge using Live behavior that is not part of the public Extensions SDK.
- `Live-plugins-1.db` is an undocumented, Ableton-owned internal schema. The importer opens it read-only, copies normalized metadata into the palette database, and must fail safely if the schema changes.
- Compatibility-sensitive code must remain isolated and version-checked. Ableton Live updates may require bridge changes.
- Custom MIDI Remote Scripts are user-installable but are not technically supported by Ableton.

## Product and data notice

- The product is local-first and should bind integration services to loopback interfaces only.
- No telemetry or cloud service is planned.
- Authentication tokens, ports, and runtime state must not be committed to source control.
- User favorites, aliases, collections, workflows, and usage history belong to the user and should be stored in documented local locations.
- Ableton's index contents and local plug-in paths are user-machine data. They must never be committed, uploaded, or included in diagnostics by default.

## Licensing notice

- `refs/` contains reference repositories and is not product source.
- Ableton's Extensions SDK distribution must not be committed or redistributed. Its license permits building applications with it but prohibits giving away or distributing the SDK itself outside the application.
- Loungy and Clui CC were studied for command-window behavior and interaction patterns. No Loungy or Clui source or asset has been copied into the current GPUI crate.
- Loungy is licensed under the MIT License. Any substantial copied portion must retain its copyright and license notice.
- GPUI is pinned to the user's `dwrekofc/zed` fork at revision `82d88a45b46e82c5da466fed44cc96375984cc4b` with runtime shaders enabled. Revisions must be upgraded deliberately.
- Do not assume the user's personal-use permission replaces upstream license obligations for a future public release.

## Development notice

- Do not add UI dependencies to the shared core crates.
- Preserve a headless test path for every integration feature.
- The npm dependency audit is currently clean. npm may still warn that `esbuild` and optional `fsevents` install scripts are not allow-listed; these are build-time packages and no blanket script approval is required for the checked-in build.
- The `.ablx` file is a local development package. It is unsigned and intended only for the compatible Live beta on this machine.
- The first in-Live scan verified 5,000 items. A larger scan exposed and fixed a bridge write-timeout issue; the installed Remote Script must be reloaded by restarting Live before validating scans above that boundary.
- The installed official extension did not activate as a useful runtime peer during the smoke test. It produced no crash, but its current SDK coverage adds no capability needed by the working primary path.
- The command bar is a background macOS agent application with no Dock icon. It registers Command-J globally but opens only when the foreground application's bundle identifier is Ableton Live's `com.ableton.live` (with an Ableton-name fallback).
- Command-bar windows are deliberately disposable rather than merely hidden. This prevents a stale floating window from following the user into another application.
