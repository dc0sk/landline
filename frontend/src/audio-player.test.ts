import { test } from "node:test";
import assert from "node:assert/strict";
import {
  encodeAudioFrame,
  floatToPcm16,
  JitterBuffer,
  parseAudioFrame,
  type AudioFrame,
} from "./audio-player.ts";

function makeBuffer(seq: number, samples: number[]): ArrayBuffer {
  const buffer = new ArrayBuffer(8 + samples.length * 2);
  const view = new DataView(buffer);
  view.setBigUint64(0, BigInt(seq), true);
  samples.forEach((s, i) => view.setInt16(8 + i * 2, s, true));
  return buffer;
}

test("parseAudioFrame reads the sequence header and PCM samples", () => {
  const frame = parseAudioFrame(makeBuffer(42, [0, 100, -100, 32000]));
  assert.equal(frame.seq, 42);
  assert.deepEqual([...frame.pcm], [0, 100, -100, 32000]);
});

test("encode/parse round-trips a frame", () => {
  const pcm = Int16Array.of(0, 1, -1, 32000, -32000);
  const parsed = parseAudioFrame(encodeAudioFrame(9, pcm));
  assert.equal(parsed.seq, 9);
  assert.deepEqual([...parsed.pcm], [...pcm]);
});

test("floatToPcm16 scales and clamps", () => {
  assert.equal(floatToPcm16(0), 0);
  assert.equal(floatToPcm16(1), 32767);
  assert.equal(floatToPcm16(-1), -32767);
  assert.equal(floatToPcm16(2), 32767); // clamped
  assert.equal(floatToPcm16(-2), -32768); // clamped
});

function frame(seq: number): AudioFrame {
  return { seq, pcm: Int16Array.of(seq) };
}

test("jitter buffer plays in order after pre-buffering", () => {
  const jb = new JitterBuffer(2, 8);
  jb.push(frame(0));
  assert.equal(jb.pop(), null); // pre-buffering
  jb.push(frame(1));
  assert.deepEqual(jb.pop(), { kind: "data", pcm: Int16Array.of(0) });
  assert.deepEqual(jb.pop(), { kind: "data", pcm: Int16Array.of(1) });
  assert.equal(jb.pop(), null); // underrun
});

test("jitter buffer reorders out-of-order frames", () => {
  const jb = new JitterBuffer(3, 8);
  jb.push(frame(0));
  jb.push(frame(2));
  jb.push(frame(1));
  assert.deepEqual(jb.pop(), { kind: "data", pcm: Int16Array.of(0) });
  assert.deepEqual(jb.pop(), { kind: "data", pcm: Int16Array.of(1) });
  assert.deepEqual(jb.pop(), { kind: "data", pcm: Int16Array.of(2) });
});

test("jitter buffer conceals a lost frame once the backlog is deep", () => {
  const jb = new JitterBuffer(1, 2);
  jb.push(frame(0));
  assert.deepEqual(jb.pop(), { kind: "data", pcm: Int16Array.of(0) });
  jb.push(frame(2));
  jb.push(frame(3));
  assert.equal(jb.pop(), null); // waiting for the missing frame 1
  jb.push(frame(4));
  assert.deepEqual(jb.pop(), { kind: "lost" });
  assert.deepEqual(jb.pop(), { kind: "data", pcm: Int16Array.of(2) });
});

test("jitter buffer drops late frames", () => {
  const jb = new JitterBuffer(1, 8);
  jb.push(frame(5));
  assert.deepEqual(jb.pop(), { kind: "data", pcm: Int16Array.of(5) });
  jb.push(frame(4)); // already played past
  assert.equal(jb.pop(), null);
});
