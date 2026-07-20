import { test } from "node:test";
import assert from "node:assert/strict";
import { TelemetryClient } from "./telemetry-client.ts";
import type { Scheduler, TimerHandle, WebSocketLike } from "./socket.ts";

class FakeSocket implements WebSocketLike {
  onopen: (() => void) | null = null;
  onclose: (() => void) | null = null;
  onmessage: ((event: { data: string | ArrayBuffer }) => void) | null = null;
  sent: string[] = [];
  close(): void {}
  send(data: string | ArrayBuffer): void {
    if (typeof data === "string") {
      this.sent.push(data);
    }
  }
}

const noopScheduler: Scheduler = {
  set: () => 0 as TimerHandle,
  clear: () => {},
};

function audioFrameBuffer(seq: number, samples: number[]): ArrayBuffer {
  const buffer = new ArrayBuffer(8 + samples.length * 2);
  const view = new DataView(buffer);
  view.setBigUint64(0, BigInt(seq), true);
  samples.forEach((s, i) => view.setInt16(8 + i * 2, s, true));
  return buffer;
}

test("subscribes to spectrum only when onFrame is set", () => {
  const sockets: FakeSocket[] = [];
  const client = new TelemetryClient({
    url: "wss://x/ws",
    token: () => "tok",
    connect: () => {
      const s = new FakeSocket();
      sockets.push(s);
      return s;
    },
    scheduler: noopScheduler,
    onFrame: () => {},
  });
  client.start();
  const socket = sockets[0]!;
  socket.onopen!();
  assert.deepEqual(JSON.parse(socket.sent[0]!), { type: "auth", token: "tok" });

  socket.onmessage!({ data: JSON.stringify({ type: "ready", role: "operator" }) });
  const subs = socket.sent.slice(1).map((s) => JSON.parse(s));
  assert.deepEqual(subs, [{ type: "subscribe", stream: "spectrum" }]);
});

test("subscribes to both spectrum and audio when both handlers are set", () => {
  const sockets: FakeSocket[] = [];
  const frames: number[][] = [];
  const audio: number[] = [];
  const client = new TelemetryClient({
    url: "wss://x/ws",
    token: () => "tok",
    connect: () => {
      const s = new FakeSocket();
      sockets.push(s);
      return s;
    },
    scheduler: noopScheduler,
    onFrame: (f) => frames.push(f.bins),
    onAudio: (f) => audio.push(f.seq),
  });
  client.start();
  const socket = sockets[0]!;
  socket.onopen!();
  socket.onmessage!({ data: JSON.stringify({ type: "ready", role: "observer" }) });
  const subs = socket.sent.slice(1).map((s) => JSON.parse(s));
  assert.deepEqual(subs, [
    { type: "subscribe", stream: "spectrum" },
    { type: "subscribe", stream: "audio" },
  ]);

  // A spectrum (text) frame and an audio (binary) frame dispatch to their handlers.
  socket.onmessage!({
    data: JSON.stringify({
      type: "spectrum",
      seq: 1,
      sample_rate: 48000,
      center_hz: 0,
      bins: [-1, -2],
    }),
  });
  socket.onmessage!({ data: audioFrameBuffer(7, [10, -10]) });
  assert.deepEqual(frames, [[-1, -2]]);
  assert.deepEqual(audio, [7]);
});

test("surfaces server errors", () => {
  const sockets: FakeSocket[] = [];
  let seen = "";
  const client = new TelemetryClient({
    url: "wss://x/ws",
    token: () => "t",
    connect: () => {
      const s = new FakeSocket();
      sockets.push(s);
      return s;
    },
    scheduler: noopScheduler,
    onFrame: () => {},
    onError: (m) => {
      seen = m;
    },
  });
  client.start();
  sockets[0]!.onopen!();
  sockets[0]!.onmessage!({ data: JSON.stringify({ type: "error", message: "nope" }) });
  assert.equal(seen, "nope");
});

test("reconnect authenticates with the current token, not the one at construction", () => {
  // The socket reconnects for the life of the page. A token captured at
  // construction is dead after the first refresh, so every later reconnect
  // would authenticate with a stale credential and be rejected forever.
  const sockets: FakeSocket[] = [];
  let current: string | null = "first";
  const client = new TelemetryClient({
    url: "wss://x/ws",
    token: () => current,
    connect: () => {
      const s = new FakeSocket();
      sockets.push(s);
      return s;
    },
    scheduler: { set: (fn) => { fn(); return 0 as TimerHandle; }, clear: () => {} },
    onFrame: () => {},
  });
  client.start();
  sockets[0]!.onopen!();
  assert.deepEqual(JSON.parse(sockets[0]!.sent[0]!), { type: "auth", token: "first" });

  // The session refreshes, then the socket drops and reconnects.
  current = "second";
  sockets[0]!.onclose!();
  const reconnected = sockets[1]!;
  reconnected.onopen!();
  assert.deepEqual(JSON.parse(reconnected.sent[0]!), { type: "auth", token: "second" });
});

test("a reconnect with no session stops instead of looping unauthenticated", () => {
  const sockets: FakeSocket[] = [];
  let current: string | null = "tok";
  const client = new TelemetryClient({
    url: "wss://x/ws",
    token: () => current,
    connect: () => {
      const s = new FakeSocket();
      sockets.push(s);
      return s;
    },
    scheduler: { set: (fn) => { fn(); return 0 as TimerHandle; }, clear: () => {} },
    onFrame: () => {},
  });
  client.start();
  sockets[0]!.onopen!();

  current = null; // signed out
  sockets[0]!.onclose!();
  const reconnected = sockets[1]!;
  reconnected.onopen!();
  assert.deepEqual(reconnected.sent, [], "must not send an auth frame with no token");
});

test("reports the server's audio sample rate on ready", () => {
  // The client must play at the rate the server captures at. Hardcoding 48 kHz
  // pitch-shifts playback whenever the rig's codec runs at another rate.
  const sockets: FakeSocket[] = [];
  let reported: number | null = null;
  const client = new TelemetryClient({
    url: "wss://x/ws",
    token: () => "tok",
    connect: () => {
      const s = new FakeSocket();
      sockets.push(s);
      return s;
    },
    scheduler: noopScheduler,
    onReady: (rate) => {
      reported = rate;
    },
    onFrame: () => {},
  });
  client.start();
  const socket = sockets[0]!;
  socket.onopen!();
  socket.onmessage!({
    data: JSON.stringify({ type: "ready", role: "operator", audio_sample_rate: 44100 }),
  });
  assert.equal(reported, 44100);
});

test("refuses an audio codec it cannot decode instead of playing noise", () => {
  // Opus payloads reinterpreted as PCM are noise. The client must decline the
  // stream and say so, not subscribe and render whatever bytes arrive.
  const sockets: FakeSocket[] = [];
  let error = "";
  const audio: unknown[] = [];
  const client = new TelemetryClient({
    url: "wss://x/ws",
    token: () => "tok",
    connect: () => {
      const s = new FakeSocket();
      sockets.push(s);
      return s;
    },
    scheduler: noopScheduler,
    onAudio: (f) => audio.push(f),
    onError: (m) => {
      error = m;
    },
  });
  client.start();
  const socket = sockets[0]!;
  socket.onopen!();
  socket.onmessage!({
    data: JSON.stringify({
      type: "ready",
      role: "operator",
      audio_sample_rate: 48000,
      audio_codec: "opus",
    }),
  });

  const subscriptions = socket.sent.map((s) => JSON.parse(s) as Record<string, unknown>);
  assert.ok(
    !subscriptions.some((m) => m["stream"] === "audio"),
    "must not subscribe to a codec it cannot decode",
  );
  assert.match(error, /opus/);

  // Even if frames arrive anyway, they must not be interpreted as PCM.
  socket.onmessage!({ data: new ArrayBuffer(16) });
  assert.deepEqual(audio, []);
});

test("subscribes to audio when the codec is PCM", () => {
  // Negative control: the refusal above must be the codec check, not a broken
  // audio subscription path.
  const sockets: FakeSocket[] = [];
  const client = new TelemetryClient({
    url: "wss://x/ws",
    token: () => "tok",
    connect: () => {
      const s = new FakeSocket();
      sockets.push(s);
      return s;
    },
    scheduler: noopScheduler,
    onAudio: () => {},
  });
  client.start();
  const socket = sockets[0]!;
  socket.onopen!();
  socket.onmessage!({
    data: JSON.stringify({
      type: "ready",
      role: "operator",
      audio_sample_rate: 48000,
      audio_codec: "pcm",
    }),
  });
  const subscriptions = socket.sent.map((s) => JSON.parse(s) as Record<string, unknown>);
  assert.ok(subscriptions.some((m) => m["stream"] === "audio"));
});
