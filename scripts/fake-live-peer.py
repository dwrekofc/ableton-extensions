#!/usr/bin/env python3
"""Protocol-compatible fake Live peer for automated daemon/CLI tests."""

import argparse
import json
import os
import socket
import sys


def parse_args():
    parser = argparse.ArgumentParser()
    parser.add_argument("--data-dir", required=True)
    parser.add_argument("--role", choices=("bridge", "extension"), required=True)
    return parser.parse_args()


def response(request_id, result=None, error=None):
    return {
        "type": "response",
        "request_id": request_id,
        "result": result,
        "error": error,
    }


def bridge_result(request):
    method = request["request"]["method"]
    params = request["request"].get("params") or {}
    if method == "get_context":
        return {
            "kind": "context",
            "data": {
                "live_version": "fake-12.4",
                "set_name": "Protocol Test",
                "track_name": "Audio 1",
                "track_kind": "audio",
                "track_index": 0,
                "selected_device_index": 0,
                "selected_device_name": "EQ Eight",
                "frozen": False,
                "devices": [
                    {
                        "index": 0,
                        "name": "EQ Eight",
                        "class_name": "Eq8Device",
                        "selected": True,
                    }
                ],
            },
        }
    if method == "inspect_devices":
        return {
            "kind": "device_inventory",
            "data": {
                "live_version": "fake-12.4",
                "set_name": "Protocol Test",
                "query": params["query"],
                "instances": [
                    {
                        "track_name": "Audio 1",
                        "track_kind": "audio",
                        "track_index": 0,
                        "device_name": "CTZ Swiss Army Meter",
                        "class_name": "MxDeviceAudioEffect",
                        "class_display_name": "Max Audio Effect",
                        "active": True,
                        "device_indices": [1, 0],
                        "chain_names": ["Meters"],
                        "device_path": [
                            "Meter Rack",
                            "Meters",
                            "CTZ Swiss Army Meter",
                        ],
                    }
                ],
            },
        }
    if method == "scan_catalog":
        return {
            "kind": "catalog_scan",
            "data": {"scanned": 1},
        }
    if method == "execute_workflow":
        actions = params["workflow"]["actions"]
        return {
            "kind": "action_results",
            "data": [
                {
                    "index": index,
                    "action_type": action["type"],
                    "success": True,
                    "message": "fake completed",
                }
                for index, action in enumerate(actions)
            ],
        }
    if method == "diagnostics":
        return {
            "kind": "diagnostics",
            "data": {
                "component": "fake_bridge",
                "version": "0.1.0",
                "live_version": "fake-12.4",
                "capabilities": ["protocol_test"],
                "warnings": [],
                "details": {},
            },
        }
    if method in ("load_item", "resolve_and_load_item", "insert_native"):
        return {"kind": "ack"}
    raise ValueError("unsupported fake bridge method: %s" % method)


def catalog_batch(request_id):
    return {
        "type": "event",
        "event": "catalog_batch",
        "data": {
            "request_id": request_id,
            "items": [
                {
                    "id": "browser:fake-reverb",
                    "kind": "native_device",
                    "source": "live_browser",
                    "name": "Reverb",
                    "aliases": [],
                    "categories": ["Audio Effects"],
                    "tags": ["space"],
                    "browser_path": ["Audio Effects", "Reverb"],
                    "compatible_tracks": ["audio", "midi", "return"],
                    "favorite": False,
                    "pinned": False,
                    "usage_count": 0,
                    "last_used_at": None,
                }
            ],
        },
    }


def main():
    args = parse_args()
    with open(os.path.join(args.data_dir, "config.json"), "r", encoding="utf-8") as handle:
        config = json.load(handle)
    sock = socket.create_connection((config["host"], int(config["port"])), timeout=5)
    stream = sock.makefile("rwb")
    hello = {
        "type": "hello",
        "protocol_version": 1,
        "role": args.role,
        "token": config["token"],
        "peer_name": "fake-%s" % args.role,
        "peer_version": "0.1.0",
        "capabilities": ["protocol_test"],
    }
    stream.write((json.dumps(hello) + "\n").encode("utf-8"))
    stream.flush()
    ack = json.loads(stream.readline())
    if ack.get("type") != "hello_ack":
        raise RuntimeError("daemon rejected fake peer")
    while True:
        line = stream.readline()
        if not line:
            return
        message = json.loads(line)
        if message.get("type") != "request":
            continue
        try:
            if args.role == "bridge":
                if message["request"]["method"] == "scan_catalog":
                    stream.write((json.dumps(catalog_batch(message["request_id"])) + "\n").encode("utf-8"))
                    stream.flush()
                result = bridge_result(message)
            elif message["request"]["method"] == "sdk_insert_native":
                result = {"kind": "ack"}
            else:
                raise ValueError("unsupported extension method")
            outgoing = response(message["request_id"], result=result)
        except Exception as exc:
            outgoing = response(
                message["request_id"],
                error={"code": "fake_error", "message": str(exc), "retryable": False},
            )
        stream.write((json.dumps(outgoing) + "\n").encode("utf-8"))
        stream.flush()


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        sys.exit(0)
