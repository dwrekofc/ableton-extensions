import {
  initialize,
  Track,
  type ActivationContext,
  type Handle,
} from "@ableton-extensions/sdk";
import { DaemonClient, type ProtocolRequest } from "./daemon-client.js";
import { sdkInsertionIndex, type InsertionPosition } from "./placement.js";

let daemon: DaemonClient | null = null;

export function activate(activation: ActivationContext) {
  const context = initialize(activation, "1.0.0");
  let targetTrack: Track<"1.0.0"> | null = null;
  daemon = new DaemonClient();

  context.commands.registerCommand("ableton-command-palette.set-target", (argument: unknown) => {
    const track = context.getObjectFromHandle(argument as Handle, Track);
    targetTrack = track;
    daemon?.send({
      type: "event",
      event: "extension_target_changed",
      data: {
        track_name: track.name,
        devices: track.devices.map((device, index) => ({ index, name: device.name })),
      },
    });
  });

  for (const scope of ["AudioTrack", "MidiTrack"] as const) {
    void context.ui.registerContextMenuAction(
      scope,
      "Use as Command Palette Target",
      "ableton-command-palette.set-target",
    );
  }

  daemon.on("request", (message: ProtocolRequest) => {
    if (message.request.method !== "sdk_insert_native") {
      daemon?.send(failure(message.request_id, "unsupported_method", "unsupported extension method"));
      return;
    }
    if (!targetTrack) {
      daemon?.send(
        failure(
          message.request_id,
          "target_unavailable",
          "Right-click an audio or MIDI track and choose Use as Command Palette Target first",
          true,
        ),
      );
      return;
    }
    const params = message.request.params ?? {};
    const name = String(params.name ?? "");
    const position = String(params.position ?? "end") as InsertionPosition;
    void Promise.resolve()
      .then(async () => {
        const index = sdkInsertionIndex(position, targetTrack!.devices.length);
        await context.withinTransaction(() => targetTrack!.insertDevice(name, index));
        daemon?.send(success(message.request_id));
      })
      .catch((error: unknown) => {
        daemon?.send(
          failure(
            message.request_id,
            "sdk_error",
            error instanceof Error ? error.message : String(error),
          ),
        );
      });
  });
  daemon.start();
}

export function deactivate() {
  daemon?.stop();
  daemon = null;
}

function success(requestId: string) {
  return {
    type: "response",
    request_id: requestId,
    result: { kind: "ack" },
    error: null,
  };
}

function failure(
  requestId: string,
  code: string,
  message: string,
  retryable = false,
) {
  return {
    type: "response",
    request_id: requestId,
    result: null,
    error: { code, message, retryable },
  };
}
