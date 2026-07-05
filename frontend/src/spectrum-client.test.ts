import { test } from "node:test";
import assert from "node:assert/strict";
import { SpectrumClient } from "./spectrum-client.ts";
import type { Scheduler, TimerHandle, WebSocketLike } from "./socket.ts";

class FakeSocket implements WebSocketLike {
  onopen: (() => void) | null = null;
  onclose: (() => void) | null = null;
  onmessage: ((event: { data: string }) => void) | null = null;
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

function harness(onError?: (message: string) => void) {
  const sockets: FakeSocket[] = [];
  const frames: number[][] = [];
  const client = new SpectrumClient({
    url: "wss://x/ws",
    token: "tok",
    connect: () => {
      const socket = new FakeSocket();
      sockets.push(socket);
      return socket;
    },
    scheduler: noopScheduler,
    onFrame: (frame) => frames.push(frame.bins),
    ...(onError ? { onError } : {}),
  });
  client.start();
  return { sockets, frames, client };
}

test("sends auth on open, subscribes on ready, delivers frames", () => {
  const { sockets, frames } = harness();
  const socket = sockets[0]!;

  socket.onopen!();
  assert.deepEqual(JSON.parse(socket.sent[0]!), { type: "auth", token: "tok" });

  socket.onmessage!({ data: JSON.stringify({ type: "ready", role: "operator" }) });
  assert.deepEqual(JSON.parse(socket.sent[1]!), { type: "subscribe", stream: "spectrum" });

  socket.onmessage!({
    data: JSON.stringify({
      type: "spectrum",
      seq: 1,
      sample_rate: 48000,
      center_hz: 0,
      bins: [-10, -20, -30],
    }),
  });
  assert.deepEqual(frames, [[-10, -20, -30]]);
});

test("surfaces server errors", () => {
  let seen = "";
  const { sockets } = harness((message) => {
    seen = message;
  });
  sockets[0]!.onopen!();
  sockets[0]!.onmessage!({ data: JSON.stringify({ type: "error", message: "nope" }) });
  assert.equal(seen, "nope");
});
