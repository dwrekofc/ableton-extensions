try:
    import Live
    from _Framework.ControlSurface import ControlSurface
except ImportError:  # Allows unit tests outside Live.
    Live = None

    class ControlSurface:
        def __init__(self, c_instance=None):
            self._c_instance = c_instance

        def schedule_message(self, _delay, _callback):
            return None

        def disconnect(self):
            return None

from .bridge_client import BridgeClient
from .live_adapter import LiveAdapter
from .protocol import failure


class AbletonCommandPalette(ControlSurface):
    POLL_TICKS = 2

    def __init__(self, c_instance):
        super().__init__(c_instance)
        application = Live.Application.get_application()
        song = self.song()
        self._adapter = LiveAdapter(song, application)
        self._bridge = BridgeClient()
        self._bridge.start()
        self.schedule_message(self.POLL_TICKS, self._poll)

    def disconnect(self):
        self._bridge.stop()
        super().disconnect()

    def _poll(self):
        for message in self._bridge.poll_requests():
            request_id = message.get("request_id", "unknown")
            request = message.get("request")
            if not isinstance(request, dict):
                self._bridge.send(failure(request_id, "invalid_request", "request body is missing"))
                continue
            response = self._adapter.handle_request(request_id, request)
            if response is not None:
                self._bridge.send(response)
        for response in self._adapter.advance_scans():
            self._bridge.send(response)
        self.schedule_message(self.POLL_TICKS, self._poll)
