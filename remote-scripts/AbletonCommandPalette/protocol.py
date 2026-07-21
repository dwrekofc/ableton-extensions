import json


PROTOCOL_VERSION = 1
MAX_FRAME_BYTES = 32 * 1024 * 1024


def encode_frame(message):
    payload = json.dumps(message, separators=(",", ":"), ensure_ascii=False).encode("utf-8")
    if len(payload) > MAX_FRAME_BYTES:
        raise ValueError("outgoing protocol frame exceeds size limit")
    return payload + b"\n"


def decode_frame(payload):
    if len(payload) > MAX_FRAME_BYTES:
        raise ValueError("incoming protocol frame exceeds size limit")
    return json.loads(payload.decode("utf-8"))


def success(request_id, kind="ack", data=None):
    if kind == "ack":
        result = {"kind": "ack"}
    else:
        result = {"kind": kind, "data": data}
    return {
        "type": "response",
        "request_id": request_id,
        "result": result,
        "error": None,
    }


def failure(request_id, code, message, retryable=False):
    return {
        "type": "response",
        "request_id": request_id,
        "result": None,
        "error": {
            "code": code,
            "message": str(message),
            "retryable": bool(retryable),
        },
    }


def event(name, data=None):
    return {
        "type": "event",
        "event": str(name),
        "data": data or {},
    }
