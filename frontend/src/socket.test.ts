import { test } from "node:test";
import assert from "node:assert/strict";
import {
  ReconnectingSocket,
  type Scheduler,
  type TimerHandle,
  type WebSocketLike,
} from "./socket.ts";

/** A fake WebSocket whose lifecycle events the test drives manually. */
class FakeSocket implements WebSocketLike {
  onopen: (() => void) | null = null;
  onclose: (() => void) | null = null;
  onmessage: ((event: { data: string }) => void) | null = null;
  closed = false;
  sent: string[] = [];

  close(): void {
    this.closed = true;
  }
  send(data: string): void {
    this.sent.push(data);
  }
}

/** A scheduler that records pending callbacks so the test can fire them. */
class FakeScheduler implements Scheduler {
  pending: { handle: TimerHandle; callback: () => void; ms: number }[] = [];
  private next = 1;

  set(callback: () => void, ms: number): TimerHandle {
    const handle = this.next++;
    this.pending.push({ handle, callback, ms });
    return handle;
  }
  clear(handle: TimerHandle): void {
    this.pending = this.pending.filter((entry) => entry.handle !== handle);
  }
  fireNext(): number {
    const entry = this.pending.shift();
    assert.ok(entry, "expected a scheduled reconnect");
    entry.callback();
    return entry.ms;
  }
}

function harness() {
  const sockets: FakeSocket[] = [];
  const scheduler = new FakeScheduler();
  const socket = new ReconnectingSocket({
    url: "wss://x",
    connect: () => {
      const s = new FakeSocket();
      sockets.push(s);
      return s;
    },
    scheduler,
  });
  return { sockets, scheduler, socket };
}

test("opening resets state and attempt counter", () => {
  const { sockets, socket } = harness();
  socket.start();
  assert.equal(socket.state, "connecting");
  sockets[0]!.onopen!();
  assert.equal(socket.state, "open");
  assert.equal(socket.attempts, 0);
});

test("reconnects with exponential backoff after each drop", () => {
  const { sockets, scheduler, socket } = harness();
  socket.start();
  sockets[0]!.onopen!();

  // First drop -> reconnect scheduled at base (1 s).
  sockets[0]!.onclose!();
  assert.equal(socket.state, "closed");
  assert.equal(scheduler.fireNext(), 1_000);
  assert.equal(sockets.length, 2, "a new socket was created");

  // Second consecutive drop (no open in between) -> 2 s.
  sockets[1]!.onclose!();
  assert.equal(scheduler.fireNext(), 2_000);

  // Third -> 4 s.
  sockets[2]!.onclose!();
  assert.equal(scheduler.fireNext(), 4_000);
});

test("a successful reopen resets the backoff", () => {
  const { sockets, scheduler, socket } = harness();
  socket.start();
  sockets[0]!.onclose!();
  assert.equal(scheduler.fireNext(), 1_000); // attempt 0
  sockets[1]!.onclose!();
  assert.equal(scheduler.fireNext(), 2_000); // attempt 1
  // Reopen resets, so the next drop is back to base.
  sockets[2]!.onopen!();
  sockets[2]!.onclose!();
  assert.equal(scheduler.fireNext(), 1_000);
});

test("stop cancels any pending reconnect", () => {
  const { sockets, scheduler, socket } = harness();
  socket.start();
  sockets[0]!.onclose!();
  assert.equal(scheduler.pending.length, 1);
  socket.stop();
  assert.equal(scheduler.pending.length, 0);
  assert.equal(socket.state, "closed");
});

test("send only forwards while open", () => {
  const { sockets, socket } = harness();
  socket.start();
  socket.send("dropped"); // still connecting
  sockets[0]!.onopen!();
  socket.send("delivered");
  assert.deepEqual(sockets[0]!.sent, ["delivered"]);
});
