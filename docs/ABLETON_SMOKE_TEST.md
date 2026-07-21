# Ableton Live Smoke Test

Targets on this machine: Ableton Live Suite 12.4.3 for the Python bridge and Ableton Live Beta 12.4.5b7 for the official Extensions SDK path. Use a disposable Live Set and save first; loading and workflow actions modify the set.

## 1. Stage the build

From the repository root:

```sh
./scripts/install-dev.sh
./scripts/test-integration.sh
./scripts/start-daemon.sh
```

The installer builds and validates every component, packages the official extension, creates local runtime configuration, and installs the Remote Script into the User Library. An existing script is moved to a timestamped backup.

## 2. Enable the Python bridge

Restart Live after installation. In Live Settings, open Link, Tempo & MIDI, add a Control Surface, and choose `AbletonCommandPalette`. Its MIDI input and output can remain `None`.

Confirm the bridge from another terminal:

```sh
./scripts/status.sh
./target/release/ableton-palette context
./target/release/ableton-palette diagnostics
```

Expected: `bridge_connected` is `true`; context names the visibly selected track and device; diagnostics include `browser_catalog` and `browser_load` without warnings.

## 3. Test catalog and loading

First import and validate Ableton's existing plug-in index:

```sh
./scripts/import-live-index.sh
./scripts/smoke-test-live-index.sh
```

Expected: the summary reports enabled instruments, effects, and vendors, then verifies that a real indexed plug-in can be found through palette search. This import is read-only and does not load anything into the Set.

Select an unfrozen test track in Live, then run:

```sh
./target/release/ableton-palette scan --max-items 5000 --max-depth 12
./target/release/ableton-palette search "eq eight"
```

Copy the returned item ID and exercise loading:

```sh
./target/release/ableton-palette load ITEM_ID --position beginning
./target/release/ableton-palette load ITEM_ID --position end
```

For relative placement, select a device in Live first:

```sh
./target/release/ableton-palette load ITEM_ID --position before
./target/release/ableton-palette load ITEM_ID --position after
```

Verify each device appears in the expected location. Undo after each check. Repeat once with an installed plug-in, preset or rack, and Max for Live device returned by search.

## 4. Test direct native insertion

```sh
./target/release/ableton-palette insert-native "EQ Eight" --position end
```

Verify insertion and undo it. A frozen track should return a clear error instead of changing the set.

## 5. Test a workflow

Save and run the provided safe example after inspecting it:

```sh
./target/release/ableton-palette save-workflow examples/test-effect-chain.json
./target/release/ableton-palette run-workflow test-effect-chain
```

Verify the track is renamed and the two devices appear in order. Undo the workflow in Live.

## 6. Recheck large-catalog transport

After installing the latest Remote Script, restart Live and run:

```sh
./target/release/ableton-palette scan --max-items 20000 --max-depth 12 >/tmp/palette-large-scan.json
./scripts/status.sh
```

Expected: the scan completes, `catalog_count` reaches the returned item count, and the bridge stays connected. This verifies the separate large-write timeout added after the first smoke test exposed a truncated response.

## Optional: official extension experiment

This path is sidelined and is not required for the MVP. Authorized SDK users can build it separately with `scripts/build-extension.sh`, then either drop the resulting `.ablx` onto the Extensions page in Live Settings or run it in development mode:

```sh
cd extension
npm start -- --live "/Applications/Ableton Live 12 Beta.app"
```

Right-click an audio or MIDI track and choose **Use as Command Palette Target**. Confirm `extension_connected` becomes `true`, then run:

```sh
./target/release/ableton-palette sdk-insert-native "EQ Eight" --position end
```

Beginning and end are supported by the official SDK. Before/after deliberately return a limitation message until the SDK exposes selected-device state.

## Logs and recovery

- Daemon: `~/Library/Application Support/Ableton Command Palette/logs/daemon.log`
- Remote Script: Live's `Log.txt` under `~/Library/Preferences/Ableton/<Live version>/`
- Extension Host: `ExtensionHost.txt` in the same Live preferences directory.

If the bridge does not connect, confirm the Control Surface selection, restart Live, and inspect `Log.txt` for `AbletonCommandPalette`. If a Browser item becomes stale, scan again.

Stop the background daemon with:

```sh
./scripts/stop-daemon.sh
```

To roll back the Remote Script, remove `AbletonCommandPalette` from the Control Surface slot and move its timestamped backup back into the `Remote Scripts` directory.
