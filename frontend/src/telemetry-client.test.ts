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
  send(data: string): void {
    this.sent.push(data);
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
    token: "tok",
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
    token: "tok",
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
    token: "t",
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
