import hashlib

from .protocol import failure, success


BROWSER_ROOTS = (
    ("Sounds", "sounds"),
    ("Drums", "drums"),
    ("Instruments", "instruments"),
    ("Audio Effects", "audio_effects"),
    ("MIDI Effects", "midi_effects"),
    ("Max for Live", "max_for_live"),
    ("Plug-Ins", "plugins"),
    ("Clips", "clips"),
    ("Samples", "samples"),
    ("Packs", "packs"),
    ("User Library", "user_library"),
    ("Current Project", "current_project"),
)


class CatalogScan:
    def __init__(self, adapter, max_items, max_depth):
        self.adapter = adapter
        self.max_items = max(1, min(int(max_items), 250000))
        self.max_depth = max(1, min(int(max_depth), 64))
        self.items = []
        self.stack = []
        self.finished = False
        for label, attribute in reversed(BROWSER_ROOTS):
            root = getattr(adapter.browser, attribute, None)
            if root is not None:
                self.stack.append((root, [label], 0, label))

    def step(self, budget=128):
        processed = 0
        while self.stack and processed < budget and len(self.items) < self.max_items:
            item, path, depth, root_label = self.stack.pop()
            processed += 1
            if self.adapter.is_loadable(item):
                record = self.adapter.catalog_record(item, path, root_label)
                self.items.append(record)
                self.adapter.item_cache[record["id"]] = item
            if depth < self.max_depth and self.adapter.is_folder(item):
                children = self.adapter.children(item)
                for child in reversed(children):
                    self.stack.append(
                        (child, path + [self.adapter.item_name(child)], depth + 1, root_label)
                    )
        if not self.stack or len(self.items) >= self.max_items:
            self.finished = True
        return self.finished


class LiveAdapter:
    def __init__(self, song, application, browser=None):
        self.song = song
        self.application = application
        self.browser = browser if browser is not None else getattr(application, "browser", None)
        self.item_cache = {}
        self.scans = {}

    def handle_request(self, request_id, request):
        method = request.get("method")
        params = request.get("params") or {}
        try:
            if method == "get_context":
                return success(request_id, "context", self.context())
            if method == "scan_catalog":
                if self.browser is None:
                    return failure(request_id, "browser_unavailable", "Live Browser API is unavailable")
                self.scans[request_id] = CatalogScan(
                    self,
                    params.get("max_items", 20000),
                    params.get("max_depth", 12),
                )
                return None
            if method == "load_item":
                return self._load_response(request_id, params)
            if method == "insert_native":
                self.insert_native(params["name"], params.get("position", "after_selected"))
                return success(request_id)
            if method == "execute_workflow":
                results = self.execute_workflow(params["workflow"])
                return success(request_id, "action_results", results)
            if method == "diagnostics":
                return success(request_id, "diagnostics", self.diagnostics())
            return failure(request_id, "unsupported_method", "unsupported bridge method: %s" % method)
        except Exception as exc:
            return failure(request_id, "live_error", str(exc))

    def advance_scans(self, budget=128):
        responses = []
        for request_id, scan in list(self.scans.items()):
            if scan.step(budget):
                responses.append(success(request_id, "catalog", scan.items))
                del self.scans[request_id]
        return responses

    def context(self):
        track = self.selected_track()
        devices = list(getattr(track, "devices", ()))
        selected = getattr(getattr(track, "view", None), "selected_device", None)
        selected_index = None
        device_records = []
        for index, device in enumerate(devices):
            is_selected = device == selected
            if is_selected:
                selected_index = index
            device_records.append(
                {
                    "index": index,
                    "name": str(getattr(device, "name", "Device")),
                    "class_name": self.optional_string(getattr(device, "class_name", None)),
                    "selected": is_selected,
                }
            )
        version = None
        getter = getattr(self.application, "get_version_string", None)
        if callable(getter):
            version = str(getter())
        return {
            "live_version": version,
            "set_name": self.optional_string(getattr(self.song, "name", None)),
            "track_name": str(getattr(track, "name", "Selected Track")),
            "track_kind": self.track_kind(track),
            "track_index": self.track_index(track),
            "selected_device_index": selected_index,
            "selected_device_name": self.optional_string(getattr(selected, "name", None)),
            "frozen": bool(getattr(track, "is_frozen", False)),
            "devices": device_records,
        }

    def diagnostics(self):
        roots = [label for label, attr in BROWSER_ROOTS if getattr(self.browser, attr, None) is not None]
        warnings = []
        if self.browser is None:
            warnings.append("Live Browser API is unavailable")
        if not roots:
            warnings.append("No known Live Browser roots were found")
        return {
            "component": "ableton_python_bridge",
            "version": "0.1.0",
            "live_version": self.context().get("live_version"),
            "capabilities": [
                "live_context",
                "native_insert",
                "workflows",
            ] + (["browser_catalog", "browser_load"] if self.browser is not None else []),
            "warnings": warnings,
            "details": {"browser_roots": roots, "cached_items": len(self.item_cache)},
        }

    def selected_track(self):
        track = getattr(getattr(self.song, "view", None), "selected_track", None)
        if track is None:
            raise RuntimeError("Live has no selected track")
        return track

    def track_kind(self, track):
        if track == getattr(self.song, "master_track", None):
            return "main"
        if track in list(getattr(self.song, "return_tracks", ())):
            return "return"
        if bool(getattr(track, "is_foldable", False)):
            return "group"
        if bool(getattr(track, "has_midi_input", False)):
            return "midi"
        if bool(getattr(track, "has_audio_input", False)):
            return "audio"
        return "unknown"

    def track_index(self, track):
        tracks = list(getattr(self.song, "tracks", ()))
        try:
            return tracks.index(track)
        except ValueError:
            return None

    def insert_native(self, name, position):
        track = self.selected_track()
        if bool(getattr(track, "is_frozen", False)):
            raise RuntimeError("cannot insert a device on a frozen track")
        devices = list(getattr(track, "devices", ()))
        index = self.insertion_index(track, position, devices)
        inserter = getattr(track, "insert_device", None)
        if not callable(inserter):
            raise RuntimeError("this Live version does not expose Track.insert_device")
        inserter(str(name), index)

    def insertion_index(self, track, position, devices=None):
        devices = list(getattr(track, "devices", ())) if devices is None else devices
        if position == "beginning":
            return 0
        if position == "end":
            return len(devices)
        selected = getattr(getattr(track, "view", None), "selected_device", None)
        try:
            index = devices.index(selected)
        except ValueError:
            return len(devices)
        if position == "before_selected":
            return index
        return index + 1

    def configure_browser_insertion(self, position):
        track = self.selected_track()
        devices = list(getattr(track, "devices", ()))
        track_view = getattr(track, "view", None)
        if track_view is None:
            raise RuntimeError("selected track view is unavailable")
        if position == "end" or not devices:
            track_view.device_insert_mode = 0
        elif position == "beginning":
            selector = getattr(getattr(self.song, "view", None), "select_device", None)
            if callable(selector):
                selector(devices[0])
            track_view.device_insert_mode = 1
        elif position == "before_selected":
            track_view.device_insert_mode = 1
        else:
            track_view.device_insert_mode = 2

    def _load_response(self, request_id, params):
        if self.browser is None:
            return failure(request_id, "browser_unavailable", "Live Browser API is unavailable")
        item_id = params["item_id"]
        item = self.item_cache.get(item_id)
        if item is None:
            item = self.resolve_path(params.get("browser_path") or [])
        if item is None:
            return failure(request_id, "item_not_found", "Browser item could not be resolved; rescan catalog")
        self.configure_browser_insertion(params.get("position", "after_selected"))
        loader = getattr(self.browser, "load_item", None)
        if not callable(loader):
            return failure(request_id, "browser_unavailable", "Live Browser load_item is unavailable")
        loader(item)
        return success(request_id)

    def execute_workflow(self, workflow):
        results = []
        for index, action in enumerate(workflow.get("actions", [])):
            action_type = action.get("type", "unknown")
            try:
                if action_type == "load_item":
                    response = self._load_response("workflow", action)
                    if response.get("error"):
                        raise RuntimeError(response["error"]["message"])
                elif action_type == "insert_native":
                    self.insert_native(action["name"], action.get("position", "end"))
                elif action_type == "set_parameter":
                    self.set_parameter(action["device_name"], action["parameter_name"], action["value"])
                elif action_type == "rename_track":
                    self.selected_track().name = action["name"]
                elif action_type == "set_track_arm":
                    self.selected_track().arm = bool(action["armed"])
                elif action_type == "set_track_mute":
                    self.selected_track().mute = bool(action["muted"])
                elif action_type == "set_track_solo":
                    self.selected_track().solo = bool(action["soloed"])
                elif action_type == "create_audio_track":
                    self.song.create_audio_track(-1)
                elif action_type == "create_midi_track":
                    self.song.create_midi_track(-1)
                else:
                    raise RuntimeError("unsupported workflow action: %s" % action_type)
                results.append(self.action_result(index, action_type, True, "completed"))
            except Exception as exc:
                results.append(self.action_result(index, action_type, False, str(exc)))
                break
        return results

    def set_parameter(self, device_name, parameter_name, value):
        devices = list(getattr(self.selected_track(), "devices", ()))
        device = next(
            (
                candidate
                for candidate in reversed(devices)
                if str(getattr(candidate, "name", "")).lower() == str(device_name).lower()
                or str(getattr(candidate, "class_display_name", "")).lower()
                == str(device_name).lower()
            ),
            None,
        )
        if device is None:
            raise RuntimeError("device not found: %s" % device_name)
        parameter = next(
            (
                candidate
                for candidate in getattr(device, "parameters", ())
                if str(getattr(candidate, "name", "")).lower() == str(parameter_name).lower()
            ),
            None,
        )
        if parameter is None:
            raise RuntimeError("parameter not found: %s" % parameter_name)
        parameter.value = float(value)

    def resolve_path(self, path):
        if not path or self.browser is None:
            return None
        root = next(
            (
                getattr(self.browser, attr, None)
                for label, attr in BROWSER_ROOTS
                if label == path[0]
            ),
            None,
        )
        if root is None:
            return None
        current = root
        for segment in path[1:]:
            current = next(
                (child for child in self.children(current) if self.item_name(child) == segment),
                None,
            )
            if current is None:
                return None
        return current

    def catalog_record(self, item, path, root_label):
        item_id = "browser:" + hashlib.sha256("\x1f".join(path).encode("utf-8")).hexdigest()
        return {
            "id": item_id,
            "kind": self.classify_item(item, path, root_label),
            "source": "live_browser",
            "name": self.item_name(item),
            "aliases": [],
            "categories": path[:-1],
            "tags": [],
            "browser_path": path,
            "compatible_tracks": [],
            "favorite": False,
            "pinned": False,
            "usage_count": 0,
            "last_used_at": None,
        }

    def classify_item(self, item, path, root_label):
        lower_name = self.item_name(item).lower()
        if root_label == "Plug-Ins":
            return "plugin"
        if root_label == "Max for Live" or lower_name.endswith(".amxd"):
            return "max_device"
        if lower_name.endswith(".adg"):
            return "rack"
        if lower_name.endswith((".adv", ".aupreset", ".vstpreset")):
            return "preset"
        if root_label == "Samples":
            return "sample"
        if bool(getattr(item, "is_device", False)):
            return "native_device"
        if root_label in ("Instruments", "Audio Effects", "MIDI Effects") and len(path) <= 2:
            return "native_device"
        return "preset"

    @staticmethod
    def item_name(item):
        return str(getattr(item, "name", "Unnamed"))

    @staticmethod
    def is_loadable(item):
        return bool(getattr(item, "is_loadable", False))

    @staticmethod
    def is_folder(item):
        return bool(getattr(item, "is_folder", False))

    @staticmethod
    def children(item):
        try:
            return list(getattr(item, "children", ()) or ())
        except Exception:
            return []

    @staticmethod
    def optional_string(value):
        return None if value is None else str(value)

    @staticmethod
    def action_result(index, action_type, successful, message):
        return {
            "index": index,
            "action_type": action_type,
            "success": successful,
            "message": message,
        }
