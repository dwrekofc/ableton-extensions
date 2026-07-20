import os
import sys
import unittest

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if ROOT not in sys.path:
    sys.path.insert(0, ROOT)

from AbletonCommandPalette.live_adapter import LiveAdapter


class FakeView:
    def __init__(self, selected_device=None, selected_track=None):
        self.selected_device = selected_device
        self.selected_track = selected_track
        self.device_insert_mode = 0

    def select_device(self, device):
        self.selected_device = device


class FakeParameter:
    def __init__(self, name, value=0.0):
        self.name = name
        self.value = value


class FakeDevice:
    def __init__(self, name):
        self.name = name
        self.class_name = name.replace(" ", "")
        self.class_display_name = name
        self.parameters = [FakeParameter("Amount")]


class FakeTrack:
    def __init__(self):
        self.name = "Audio 1"
        self.devices = [FakeDevice("EQ Eight"), FakeDevice("Compressor")]
        self.view = FakeView(self.devices[0])
        self.has_audio_input = True
        self.has_midi_input = False
        self.is_foldable = False
        self.is_frozen = False
        self.insertions = []

    def insert_device(self, name, index):
        self.insertions.append((name, index))


class FakeSong:
    def __init__(self, track):
        self.name = "Test Set"
        self.tracks = [track]
        self.return_tracks = []
        self.master_track = object()
        self.view = FakeView(selected_track=track)
        self.created = []

    def create_audio_track(self, index):
        self.created.append(("audio", index))

    def create_midi_track(self, index):
        self.created.append(("midi", index))


class FakeApplication:
    def get_version_string(self):
        return "12.4.5b7"


class FakeBrowserItem:
    def __init__(self, name, loadable=False, children=None, is_device=False):
        self.name = name
        self.is_loadable = loadable
        self.children = children or []
        self.is_folder = bool(children)
        self.is_device = is_device


class FakeBrowser:
    def __init__(self):
        self.reverb = FakeBrowserItem("Reverb", True, is_device=True)
        self.audio_effects = FakeBrowserItem("Audio Effects", children=[self.reverb])
        self.loaded = []

    def load_item(self, item):
        self.loaded.append(item)


class LiveAdapterTests(unittest.TestCase):
    def setUp(self):
        self.track = FakeTrack()
        self.song = FakeSong(self.track)
        self.browser = FakeBrowser()
        self.adapter = LiveAdapter(self.song, FakeApplication(), self.browser)

    def test_context_reports_selected_target(self):
        context = self.adapter.context()
        self.assertEqual(context["track_name"], "Audio 1")
        self.assertEqual(context["track_kind"], "audio")
        self.assertEqual(context["selected_device_index"], 0)
        self.assertEqual(context["live_version"], "12.4.5b7")

    def test_native_insertion_positions(self):
        self.adapter.insert_native("Reverb", "beginning")
        self.adapter.insert_native("Auto Filter", "before_selected")
        self.adapter.insert_native("Delay", "after_selected")
        self.adapter.insert_native("Utility", "end")
        self.assertEqual(
            self.track.insertions,
            [("Reverb", 0), ("Auto Filter", 0), ("Delay", 1), ("Utility", 2)],
        )

    def test_catalog_scan_and_load(self):
        response = self.adapter.handle_request(
            "scan-1",
            {"method": "scan_catalog", "params": {"max_items": 100, "max_depth": 4}},
        )
        self.assertIsNone(response)
        responses = self.adapter.advance_scans(100)
        self.assertEqual(len(responses), 1)
        item = responses[0]["result"]["data"][0]
        self.assertEqual(item["name"], "Reverb")
        load = self.adapter.handle_request(
            "load-1",
            {
                "method": "load_item",
                "params": {
                    "item_id": item["id"],
                    "browser_path": item["browser_path"],
                    "position": "after_selected",
                },
            },
        )
        self.assertIsNone(load["error"])
        self.assertEqual(self.browser.loaded, [self.browser.reverb])
        self.assertEqual(self.track.view.device_insert_mode, 2)

    def test_workflow_stops_after_failure(self):
        workflow = {
            "actions": [
                {"type": "rename_track", "name": "Vocal"},
                {
                    "type": "set_parameter",
                    "device_name": "Missing",
                    "parameter_name": "Amount",
                    "value": 0.5,
                },
                {"type": "set_track_mute", "muted": True},
            ]
        }
        results = self.adapter.execute_workflow(workflow)
        self.assertEqual(self.track.name, "Vocal")
        self.assertEqual(len(results), 2)
        self.assertFalse(results[-1]["success"])


if __name__ == "__main__":
    unittest.main()
