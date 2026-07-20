import json
import os
import sys


def data_directory():
    override = os.environ.get("ABLETON_PALETTE_DATA_DIR")
    if override:
        return os.path.abspath(os.path.expanduser(override))
    home = os.path.expanduser("~")
    if sys.platform == "darwin":
        return os.path.join(home, "Library", "Application Support", "Ableton Command Palette")
    if sys.platform.startswith("win"):
        return os.path.join(os.environ.get("APPDATA", home), "Ableton Command Palette")
    return os.path.join(home, ".local", "share", "Ableton Command Palette")


def load_config():
    path = os.path.join(data_directory(), "config.json")
    with open(path, "r", encoding="utf-8") as handle:
        config = json.load(handle)
    required = ("host", "port", "token", "protocol_version")
    missing = [key for key in required if key not in config]
    if missing:
        raise ValueError("palette config missing: " + ", ".join(missing))
    if config["host"] not in ("127.0.0.1", "localhost", "::1"):
        raise ValueError("palette bridge only permits loopback hosts")
    return config
