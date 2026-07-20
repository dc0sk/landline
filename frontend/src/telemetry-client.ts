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
  /**
   * Supplies the current access token. A getter, not a value: the socket
   * reconnects for the life of the page, and a token captured at construction
   * is dead after the first refresh — every later reconnect would then
   * authenticate with a stale credential and be rejected, forever.
   */
  readonly token: () => string | null;
  readonly connect: (url: string) => WebSocketLike;
  /** Called on `ready` with the server's audio sample rate (Hz). */
  readonly onReady?: (audioSampleRate: number) => void;
  readonly onFrame?: (frame: SpectrumFrame) => void;
  readonly onAudio?: (frame: AudioFrame) => void;
  readonly onError?: (message: string) => void;
  readonly scheduler?: Scheduler;
}

export class TelemetryClient {
  private readonly socket: ReconnectingSocket;
  private readonly token: () => string | null;
  private readonly onReady: ((audioSampleRate: number) => void) | undefined;
  private readonly onFrame: ((frame: SpectrumFrame) => void) | undefined;
  private readonly onAudio: ((frame: AudioFrame) => void) | undefined;
  private readonly onError: ((message: string) => void) | undefined;
  /** Set from the `ready` frame; false until the server names a codec we decode. */
  private audioCodecSupported = false;

  constructor(options: TelemetryClientOptions) {
    this.token = options.token;
    this.onReady = options.onReady;
    this.onFrame = options.onFrame;
    this.onAudio = options.onAudio;
    this.onError = options.onError;
    this.socket = new ReconnectingSocket({
      url: options.url,
      connect: options.connect,
      ...(options.scheduler ? { scheduler: options.scheduler } : {}),
      onStateChange: (state) => {
        if (state === "open") {
          const token = this.token();
          if (token === null) {
            // No session left: stop rather than reconnect-loop unauthenticated.
            this.socket.stop();
            return;
          }
          this.socket.send(JSON.stringify({ type: "auth", token }));
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
      // Defensive: never interpret bytes we did not agree a codec for.
      if (!this.audioCodecSupported) {
        return;
      }
      const frame = parseAudioFrame(data);
      if (frame !== null) {
        this.onAudio?.(frame);
      }
      return;
    }
    let message: Record<string, unknown>;
    try {
      message = JSON.parse(data) as Record<string, unknown>;
    } catch {
      return;
    }
    switch (message["type"]) {
      case "ready": {
        this.onReady?.(Number(message["audio_sample_rate"]));
        // Subscribe to exactly the streams the caller asked for.
        if (this.onFrame) {
          this.socket.send(JSON.stringify({ type: "subscribe", stream: "spectrum" }));
        }
        // Only subscribe to audio if we can actually decode the wire codec.
        // This browser build handles raw PCM only; subscribing to Opus and
        // reinterpreting its bytes as PCM produces noise, not audio.
        const codec = String(message["audio_codec"] ?? "pcm");
        this.audioCodecSupported = codec === "pcm";
        if (this.onAudio) {
          if (this.audioCodecSupported) {
            this.socket.send(JSON.stringify({ type: "subscribe", stream: "audio" }));
          } else {
            this.onError?.(
              `server audio codec "${codec}" is not supported by this client; audio disabled`,
            );
          }
        }
        break;
      }
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
