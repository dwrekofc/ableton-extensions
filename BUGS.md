# Bug and Interaction Ledger

This file records user-observed product defects before implementation work begins. Keep each item current through verification.

## ACP-001 — Indexed VST is searchable but cannot be added

- **Status:** Implemented; awaiting in-Live verification
- **Observed:** Searching `trackspa` selects `Trackspacer 2.5` from Live's plug-in index, but execution reports `Device Trackspacer 2.5 not found` even though the plug-in is visibly present in Live's Plug-Ins Browser.
- **Expected outcome:** Selecting an installed VST/VST3 loads the matching Live Browser item at the chosen chain position.
- **Likely boundary:** Ableton's database entry has no resolved Browser path, and direct `Track.insert_device(name)` does not load this third-party plug-in.
- **Acceptance:** `Trackspacer 2.5` resolves by exact Browser name and loads without requiring the user to navigate the Browser.
- **Resolution:** Database-only results are now transformed by the daemon into a Browser-resolution request. The Remote Script searches the Plug-Ins root first, caches the resolved Live object under the indexed ID, applies the selected insertion mode, and loads through `Browser.load_item`.

## ACP-002 — Results stop at twelve and cannot be scrolled

- **Status:** Implemented; awaiting visual verification
- **Observed:** The palette reports twelve matches, clips the final row against the footer, and offers no access to additional matches.
- **Expected outcome:** Search can return a useful result set; mouse/trackpad scrolling and keyboard navigation reach every returned item, with the selected row kept visible above the footer.
- **Acceptance:** More than twelve results render in a contained scrolling region and Up/Down navigation scrolls the active row into view.
- **Resolution:** Search now requests up to 200 ranked matches. The body owns a tracked vertical scroll region, renders every returned result, and scrolls the selected result into view during keyboard navigation.

## ACP-003 — Escape/focus loss leaves a visible palette window

- **Status:** Implemented; awaiting visual verification
- **Observed:** After Escape, the palette may remain visible as another window and can follow the user into a different application.
- **Expected outcome:** Escape fully hides the agent window. Switching away from the palette also hides it. It cannot appear again until Ableton is frontmost and Command-J is pressed.
- **Acceptance:** No palette pixels remain after Escape or application switching, Command-J is ignored outside Ableton, and returning to Ableton does not reopen it without a fresh Command-J.
- **Resolution:** Escape and window deactivation now remove the GPUI window entirely and clear its global handle. The resident agent has no window until a new eligible Command-J event creates one.

## ACP-004 — Search results lack content-category grouping

- **Status:** Implemented; awaiting visual verification
- **Observed:** Results form one undifferentiated list.
- **Expected outcome:** Small headers separate samples/loops, devices, plug-ins, presets, racks, Max for Live, commands, and workflows without interrupting keyboard navigation.
- **Acceptance:** Every visible result appears under a stable content-group header and selection skips headers.
- **Resolution:** Results retain their fuzzy rank inside stable device, plug-in, preset, rack, Max for Live, sample, loop/clip, command, workflow, and other groups. Headers are non-selectable children of the result scroller.

## ACP-005 — Selected items have no action menu

- **Status:** Implemented; awaiting interaction verification
- **Observed:** There is no keyboard route to act on the selected catalog item without executing it.
- **Expected outcome:** Command-K opens an item-action surface for the selected result. The first supported action toggles Favorite.
- **Acceptance:** Command-K opens the action surface, Enter toggles the selected item's favorite state, and Escape returns to search without closing the palette.
- **Resolution:** Command-K switches to an item-action view for the selected result. Its Favorite action calls the existing persisted daemon preference API and refreshes search after success.

## ACP-006 — Search content groups cannot be hidden

- **Status:** Implemented; awaiting interaction verification
- **Observed:** All catalog kinds always participate in search.
- **Expected outcome:** Command-Comma opens settings where users can show or hide samples/loops, devices, plug-ins, presets, racks, Max for Live, commands, and workflows. Choices persist between launches.
- **Acceptance:** Toggling a group immediately filters results and persists after restarting the command bar.
- **Resolution:** Command-Comma opens a keyboard-and-mouse settings view backed by local `ui-settings.json`. Each group can be shown or hidden independently and changes immediately re-run the current search.
