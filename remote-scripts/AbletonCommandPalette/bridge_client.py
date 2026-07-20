import logging
import queue
import socket
import threading
import time

from .config import load_config
from .protocol import PROTOCOL_VERSION, decode_frame, encode_frame


class BridgeClient:
    """Network transport. This thread never accesses a Live object."""

    def __init__(self, peer_version="0.1.0"):
        self.peer_version = peer_version
        self.incoming = queue.Queue()
        self.outgoing = queue.Queue()
        self._stop = threading.Event()
        self._connected = threading.Event()
        self._thread = None
        self._logger = logging.getLogger("AbletonCommandPalette.bridge")

    @property
    def connected(self):
        return self._connected.is_set()

    def start(self):
        if self._thread and self._thread.is_alive():
            return
        self._stop.clear()
        self._thread = threading.Thread(
            target=self._run,
            name="AbletonCommandPaletteBridge",
            daemon=True,
        )
        self._thread.start()

    def stop(self):
        self._stop.set()
        if self._thread and self._thread.is_alive():
            self._thread.join(timeout=1.0)
        self._connected.clear()

    def poll_requests(self, limit=32):
        messages = []
        for _ in range(limit):
            try:
                messages.append(self.incoming.get_nowait())
            except queue.Empty:
                break
        return messages

    def send(self, message):
        self.outgoing.put(message)

    def _run(self):
        backoff = 0.5
        while not self._stop.is_set():
            sock = None
            try:
                config = load_config()
                sock = socket.create_connection((config["host"], int(config["port"])), timeout=2.0)
                sock.settimeout(0.2)
                hello = {
                    "type": "hello",
                    "protocol_version": PROTOCOL_VERSION,
                    "role": "bridge",
                    "token": config["token"],
                    "peer_name": "ableton-python-remote-script",
                    "peer_version": self.peer_version,
                    "capabilities": [
                        "live_context",
                        "browser_catalog",
                        "browser_load",
                        "native_insert",
                        "workflows",
                    ],
                }
                sock.sendall(encode_frame(hello))
                buffer = b""
                deadline = time.time() + 5.0
                while time.time() < deadline:
                    frames, buffer = self._receive(sock, buffer)
                    if any(frame.get("type") == "hello_ack" for frame in frames):
                        break
                else:
                    raise RuntimeError("daemon did not acknowledge bridge")

                self._connected.set()
                backoff = 0.5
                while not self._stop.is_set():
                    self._flush(sock)
                    frames, buffer = self._receive(sock, buffer)
                    for frame in frames:
                        if frame.get("type") == "request":
                            self.incoming.put(frame)
            except Exception as exc:
                self._logger.debug("bridge disconnected: %s", exc)
            finally:
                self._connected.clear()
                if sock:
                    try:
                        sock.close()
                    except OSError:
                        pass
            self._stop.wait(backoff)
            backoff = min(backoff * 2.0, 5.0)

    def _flush(self, sock):
        while True:
            try:
                message = self.outgoing.get_nowait()
            except queue.Empty:
                return
            previous_timeout = sock.gettimeout()
            try:
                # Large catalog responses can exceed the short receive polling timeout.
                # Give loopback writes enough time to finish rather than disconnecting
                # mid-frame and leaving the daemon with truncated JSON.
                sock.settimeout(30.0)
                sock.sendall(encode_frame(message))
            finally:
                sock.settimeout(previous_timeout)

    @staticmethod
    def _receive(sock, buffer):
        try:
            chunk = sock.recv(65536)
            if not chunk:
                raise ConnectionError("daemon closed connection")
            buffer += chunk
        except socket.timeout:
            return [], buffer
        frames = []
        while b"\n" in buffer:
            payload, buffer = buffer.split(b"\n", 1)
            if payload:
                frames.append(decode_frame(payload))
        return frames, buffer
