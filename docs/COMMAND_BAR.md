# Command Bar

## Outcome

The native command bar makes the indexed Ableton Browser available without leaving the selected track. It appears only from Ableton Live or Ableton Live Beta, searches the same local catalog used by the CLI, and sends the chosen result back to Live.

## Start

```sh
./scripts/start-command-bar.sh
```

The script starts the local service, builds and installs the lightweight macOS app in the project's application-data folder, and opens it without adding a Dock icon.

Focus Live and press `Command-J`. The shortcut closes the bar when it is already visible. It does nothing from other applications.

## Keyboard controls

- Type to search names, aliases, categories, and tags.
- `Up` / `Down` selects a result.
- `Enter` adds or runs the selected result.
- Mouse/trackpad scrolling and keyboard navigation can traverse up to 200 ranked results; keyboard selection is always scrolled into view.
- `Tab` / `Shift-Tab` cycles beginning, before selected, after selected, and end placement.
- `Command-K` opens actions for the selected item. The first action adds or removes it from Favorites.
- `Command-Comma` opens content settings. Use Up/Down plus Space/Enter—or click—to show or hide devices, plug-ins, presets, racks, Max for Live, samples, loops/clips, commands, and workflows.
- `Command-R` refreshes every available Live Browser root into the catalog in safe batches.
- `Escape` returns from settings/actions or fully closes the bar from search. The GPUI window is destroyed when closed or when focus moves elsewhere, and is recreated only by Command-J from Live.

## Search and execution behavior

Live Browser results retain their Browser path and load through Live's Browser API. For plug-ins discovered only through Live's internal index, the Remote Script searches Live's Plug-Ins Browser by exact name and loads the resolved Browser object. This avoids the native-device insertion API, which does not accept many third-party VST names.

Results are grouped by content type with non-selectable headers. Fuzzy relevance and favorite/pin/usage ranking remain intact within each group.

The footer shows the selected track and selected device so placement can be confirmed before execution. A frozen or incompatible track is rejected by Live with an explanatory message.

## Refresh requirement

The batched full-Browser scanner requires the current Remote Script. Run `./scripts/install-remote-script.sh` and restart Live after an update. Existing catalog search and loading continue to work before a refresh.

Content visibility preferences are stored locally in `ui-settings.json` beside the palette database. All groups are visible by default.

## Stop

```sh
./scripts/stop-command-bar.sh
```
