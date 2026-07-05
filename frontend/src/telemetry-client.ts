// Telemetry WebSocket client (ARC-10 → ARC-01, A28/A31). One authenticated,
// reconnecting socket multiplexing spectrum (JSON text frames) and audio (binary
// frames) per ADR-02. Handshake: send the JWT `auth` frame on open, subscribe to
// the streams the caller wants, and dispatch each frame to its callback.

import { parseAudioFrame, type AudioFrame } from "./audio-player.ts";
import { ReconnectingSocket, type Scheduler, type WebSocketLike } from "./socket.ts";

export interface SpectrumFrame {
  readonly seq: number;
  readonly sampleRate: number;
  readonly centerHz: number;
  readonly bins: number[];
}

export interface TelemetryClientOptions {
  readonly url: string;
  readonly token: string;
  readonly connect: (url: string) => WebSocketLike;
  readonly onFrame?: (frame: SpectrumFrame) => void;
  readonly onAudio?: (frame: AudioFrame) => void;
  readonly onError?: (message: string) => void;
  readonly scheduler?: Scheduler;
}

export class TelemetryClient {
  private readonly socket: ReconnectingSocket;
  private readonly token: string;
  private readonly onFrame: ((frame: SpectrumFrame) => void) | undefined;
  private readonly onAudio: ((frame: AudioFrame) => void) | undefined;
  private readonly onError: ((message: string) => void) | undefined;

  constructor(options: TelemetryClientOptions) {
    this.token = options.token;
    this.onFrame = options.onFrame;
    this.onAudio = options.onAudio;
    this.onError = options.onError;
    this.socket = new ReconnectingSocket({
      url: options.url,
      connect: options.connect,
      ...(options.scheduler ? { scheduler: options.scheduler } : {}),
      onStateChange: (state) => {
        if (state === "open") {
          this.socket.send(JSON.stringify({ type: "auth", token: this.token }));
        }
      },
      onMessage: (data) => this.handle(data),
    });
  }

  start(): void {
    this.socket.start();
  }

  stop(): void {
    this.socket.stop();
  }

  /** Send an encoded mic frame to the server (TX audio, FR-AUD-02). */
  sendAudio(frame: ArrayBuffer): void {
    this.socket.send(frame);
  }

  private handle(data: string | ArrayBuffer): void {
    if (data instanceof ArrayBuffer) {
      this.onAudio?.(parseAudioFrame(data));
      return;
    }
    let message: Record<string, unknown>;
    try {
      message = JSON.parse(data) as Record<string, unknown>;
    } catch {
      return;
    }
    switch (message["type"]) {
      case "ready":
        // Subscribe to exactly the streams the caller asked for.
        if (this.onFrame) {
          this.socket.send(JSON.stringify({ type: "subscribe", stream: "spectrum" }));
        }
        if (this.onAudio) {
          this.socket.send(JSON.stringify({ type: "subscribe", stream: "audio" }));
        }
        break;
      case "spectrum":
        this.onFrame?.({
          seq: Number(message["seq"]),
          sampleRate: Number(message["sample_rate"]),
          centerHz: Number(message["center_hz"]),
          bins: Array.isArray(message["bins"]) ? (message["bins"] as number[]) : [],
        });
        break;
      case "error":
        this.onError?.(String(message["message"] ?? "error"));
        break;
      default:
        break;
    }
  }
}
