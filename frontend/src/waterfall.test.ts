import { test } from "node:test";
import assert from "node:assert/strict";
import { normalizeDb, palette, rowRgba } from "./waterfall.ts";

test("normalizeDb maps and clamps to [0, 1]", () => {
  assert.equal(normalizeDb(-100, -100, -20), 0);
  assert.equal(normalizeDb(-20, -100, -20), 1);
  assert.equal(normalizeDb(-60, -100, -20), 0.5);
  assert.equal(normalizeDb(-200, -100, -20), 0); // below range
  assert.equal(normalizeDb(0, -100, -20), 1); // above range
});

test("palette maps 0 to black and 1 to white", () => {
  assert.deepEqual(palette(0), [0, 0, 0]);
  assert.deepEqual(palette(1), [255, 255, 255]);
});

test("rowRgba yields one opaque RGBA pixel per bin", () => {
  const row = rowRgba([-100, -20], -100, -20);
  assert.equal(row.length, 8);
  assert.deepEqual([...row.slice(0, 4)], [0, 0, 0, 255]); // min -> black
  assert.deepEqual([...row.slice(4, 8)], [255, 255, 255, 255]); // max -> white
});
