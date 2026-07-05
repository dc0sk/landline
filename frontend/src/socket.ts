// Reconnecting WebSocket client (ARC-10, A20). Wraps a WebSocket-like transport
// with exponential-backoff reconnection (NFR-REL-01): on close it schedules a
// reconnect using backoff.ts, resetting the attempt counter once a connection
// opens. Transport and timer are injected so the reconnect policy is unit-
// testable without a real socket or wall-clock (TC-REL-01).
//
// Phase 1 control is REST; this client is the seam for the Phase-2 WS telemetry
// channel (spectrum, audio, live S-meter — ADR-02), wired now so the reconnect
// behaviour is proven before that transport lands.

import { backoffDelay, DEFAULT_BACKOFF, type BackoffOptions } from "./backoff.ts";

/** The minimal WebSocket surface this client depends on. */
export interface WebSocketLike {
  close(): void;
  send(data: string): void;
  onopen: (() => void) | null;
  onclose: (() => void) | null;
  onmessage: ((event: { data: string }) => void) | null;
}

export type SocketState = "connecting" | "open" | "closed";

/** Opaque timer handle (a browser timeout id). */
export type TimerHandle = number;

export interface Scheduler {
  set(callback: () => void, ms: number): TimerHandle;
  clear(handle: TimerHandle): void;
}

export interface ReconnectingSocketOptions {
  readonly url: string;
  readonly connect: (url: string) => WebSocketLike;
  readonly onMessage?: (data: string) => void;
  readonly onStateChange?: (state: SocketState) => void;
  readonly backoff?: BackoffOptions;
  readonly scheduler?: Scheduler;
}

const BROWSER_SCHEDULER: Scheduler = {
  set: (callback, ms) => window.setTimeout(callback, ms),
  clear: (handle) => {
    window.clearTimeout(handle);
  },
};

export class ReconnectingSocket {
  private readonly url: string;
  private readonly connect: (url: string) => WebSocketLike;
  private readonly onMessage: ((data: string) => void) | undefined;
  private readonly onStateChange: ((state: SocketState) => void) | undefined;
  private readonly backoff: BackoffOptions;
  private readonly scheduler: Scheduler;

  private socket: WebSocketLike | null = null;
  private timer: TimerHandle | null = null;
  private attempt = 0;
  private stopped = false;
  private currentState: SocketState = "closed";

  constructor(options: ReconnectingSocketOptions) {
    this.url = options.url;
    this.connect = options.connect;
    this.onMessage = options.onMessage;
    this.onStateChange = options.onStateChange;
    this.backoff = options.backoff ?? DEFAULT_BACKOFF;
    this.scheduler = options.scheduler ?? BROWSER_SCHEDULER;
  }

  get state(): SocketState {
    return this.currentState;
  }

  /** The number of consecutive failed connects since the last open. */
  get attempts(): number {
    return this.attempt;
  }

  /** Open the connection and keep it open across drops until `stop()`. */
  start(): void {
    this.stopped = false;
    this.open();
  }

  /** Stop reconnecting and close any open connection. */
  stop(): void {
    this.stopped = true;
    if (this.timer !== null) {
      this.scheduler.clear(this.timer);
      this.timer = null;
    }
    this.socket?.close();
    this.socket = null;
    this.setState("closed");
  }

  /** Send data if the socket is open; a no-op otherwise. */
  send(data: string): void {
    if (this.currentState === "open") {
      this.socket?.send(data);
    }
  }

  private open(): void {
    this.setState("connecting");
    const socket = this.connect(this.url);
    this.socket = socket;
    socket.onopen = () => {
      this.attempt = 0;
      this.setState("open");
    };
    socket.onmessage = (event) => this.onMessage?.(event.data);
    socket.onclose = () => {
      this.socket = null;
      this.setState("closed");
      this.scheduleReconnect();
    };
  }

  private scheduleReconnect(): void {
    if (this.stopped) {
      return;
    }
    const delay = backoffDelay(this.attempt, this.backoff);
    this.attempt += 1;
    this.timer = this.scheduler.set(() => {
      this.timer = null;
      this.open();
    }, delay);
  }

  private setState(state: SocketState): void {
    if (state !== this.currentState) {
      this.currentState = state;
      this.onStateChange?.(state);
    }
  }
}

/** Adapt a real browser `WebSocket` to [`WebSocketLike`]. */
export function browserSocket(url: string): WebSocketLike {
  const ws = new WebSocket(url);
  const like: WebSocketLike = {
    close: () => ws.close(),
    send: (data) => ws.send(data),
    onopen: null,
    onclose: null,
    onmessage: null,
  };
  ws.onopen = () => like.onopen?.();
  ws.onclose = () => like.onclose?.();
  ws.onmessage = (event) => like.onmessage?.({ data: String(event.data) });
  return like;
}
