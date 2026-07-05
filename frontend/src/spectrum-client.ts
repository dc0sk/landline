// Spectrum WebSocket client (ARC-10 → ARC-01/06, A29). Wraps the reconnecting
// socket with the telemetry handshake: on connect, send the JWT `auth` frame
// (FR-AUTH-01 — token in the body, never the URL); on `ready`, subscribe to the
// spectrum stream; deliver each `spectrum` frame to the caller.

import { ReconnectingSocket, type Scheduler, type WebSocketLike } from "./socket.ts";

export interface SpectrumFrame {
  readonly seq: number;
  readonly sampleRate: number;
  readonly centerHz: number;
  readonly bins: number[];
}

export interface SpectrumClientOptions {
  readonly url: string;
  readonly token: string;
  readonly connect: (url: string) => WebSocketLike;
  readonly onFrame: (frame: SpectrumFrame) => void;
  readonly onError?: (message: string) => void;
  readonly scheduler?: Scheduler;
}

export class SpectrumClient {
  private readonly socket: ReconnectingSocket;
  private readonly token: string;
  private readonly onFrame: (frame: SpectrumFrame) => void;
  private readonly onError: ((message: string) => void) | undefined;

  constructor(options: SpectrumClientOptions) {
    this.token = options.token;
    this.onFrame = options.onFrame;
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

  private handle(data: string): void {
    let message: Record<string, unknown>;
    try {
      message = JSON.parse(data) as Record<string, unknown>;
    } catch {
      return;
    }
    switch (message["type"]) {
      case "ready":
        this.socket.send(JSON.stringify({ type: "subscribe", stream: "spectrum" }));
        break;
      case "spectrum":
        this.onFrame({
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
