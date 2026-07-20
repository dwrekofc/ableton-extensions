# Ableton Command Palette — Experimental Official Extension

This optional path is currently sidelined. The primary product uses the Python Remote Script because it provides selection, Browser discovery/loading, every placement mode, and workflows. Nothing in the MVP requires this extension.

The source remains as a forward-compatible experiment for supported native insertion and context-menu targeting.

The current SDK does not expose Live's selected track or selected device globally. To use the SDK insertion path, right-click an audio or MIDI track and choose **Use as Command Palette Target**. Boundary placement (`beginning` or `end`) is supported. Before/after-selected placement is handled by the Python Live bridge.

Development requires the compatible Ableton Live beta with Extensions Developer Mode enabled. Ableton's SDK is not redistributed in the public repository; obtain it directly from Ableton and place its distribution at `extensions-sdk-1.0.0-beta.0/` before running `../scripts/build-extension.sh`.
