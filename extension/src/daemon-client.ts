import { EventEmitter } from "node:events";
import * as fs from "node:fs";
import * as net from "node:net";
import * as os from "node:os";
import * as path from "node:path";

const PROTOCOL_VERSION = 1;
const MAX_FRAME_BYTES = 32 * 1024 * 1024;

interface RuntimeConfig {
  host: string;
  port: number;
  token: string;
  protocol_version: number;
}

export interface ProtocolRequest {
  type: "request";
  request_id: string;
  request: {
    method: string;
    params?: Record<string, unknown>;
  };
}

export class DaemonClient extends EventEmitter {
  private socket: net.Socket | null = null;
  private buffer = "";
  private reconnectTimer: NodeJS.Timeout | null = null;
  private stopped = false;

  start(): void {
    this.stopped = false;
    this.connect();
  }

  stop(): void {
    this.stopped = true;
    if (this.reconnectTimer) clearTimeout(this.reconnectTimer);
    this.reconnectTimer = null;
    this.socket?.destroy();
    this.socket = null;
  }

  send(message: unknown): void {
    if (!this.socket?.writable) return;
    const json = JSON.stringify(message);
    if (Buffer.byteLength(json) > MAX_FRAME_BYTES) {
      throw new Error("outgoing protocol frame exceeds size limit");
    }
    this.socket.write(`${json}\n`);
  }

  private connect(): void {
    if (this.stopped || this.socket) return;
    let config: RuntimeConfig;
    try {
      config = loadConfig();
    } catch {
      this.scheduleReconnect();
      return;
    }
    if (!["127.0.0.1", "localhost", "::1"].includes(config.host)) {
      this.scheduleReconnect();
      return;
    }
    const socket = net.createConnection({ host: config.host, port: config.port });
    this.socket = socket;
    socket.setNoDelay(true);
    socket.setEncoding("utf8");
    socket.once("connect", () => {
      this.send({
        type: "hello",
        protocol_version: PROTOCOL_VERSION,
        role: "extension",
        token: config.token,
        peer_name: "official-ableton-extension",
        peer_version: "0.1.0",
        capabilities: ["context_menu_target", "sdk_native_insert"],
      });
    });
    socket.on("data", (chunk: string) => this.receive(chunk));
    socket.on("error", () => socket.destroy());
    socket.on("close", () => {
      if (this.socket === socket) this.socket = null;
      this.buffer = "";
      this.scheduleReconnect();
    });
  }

  private receive(chunk: string): void {
    this.buffer += chunk;
    if (Buffer.byteLength(this.buffer) > MAX_FRAME_BYTES) {
      this.socket?.destroy(new Error("incoming protocol frame exceeds size limit"));
      return;
    }
    while (this.buffer.includes("\n")) {
      const newline = this.buffer.indexOf("\n");
      const line = this.buffer.slice(0, newline);
      this.buffer = this.buffer.slice(newline + 1);
      if (!line) continue;
      try {
        const message = JSON.parse(line) as { type?: string };
        if (message.type === "request") this.emit("request", message as ProtocolRequest);
      } catch {
        // Ignore malformed frames; the daemon will reconnect if protocol health degrades.
      }
    }
  }

  private scheduleReconnect(): void {
    if (this.stopped || this.reconnectTimer) return;
    this.reconnectTimer = setTimeout(() => {
      this.reconnectTimer = null;
      this.connect();
    }, 1500);
  }
}

function loadConfig(): RuntimeConfig {
  const override = process.env.ABLETON_PALETTE_DATA_DIR;
  const dataDirectory = override
    ? path.resolve(override)
    : process.platform === "darwin"
      ? path.join(os.homedir(), "Library", "Application Support", "Ableton Command Palette")
      : process.platform === "win32"
        ? path.join(process.env.APPDATA || os.homedir(), "Ableton Command Palette")
        : path.join(os.homedir(), ".local", "share", "Ableton Command Palette");
  return JSON.parse(
    fs.readFileSync(path.join(dataDirectory, "config.json"), "utf8"),
  ) as RuntimeConfig;
}
