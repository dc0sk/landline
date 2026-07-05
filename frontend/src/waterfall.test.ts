import { test } from "node:test";
import assert from "node:assert/strict";
import {
  grayscalePalette,
  hotPalette,
  icePalette,
  normalizeDb,
  PALETTE_NAMES,
  rowRgba,
} from "./waterfall.ts";

test("normalizeDb maps and clamps to [0, 1]", () => {
  assert.equal(normalizeDb(-100, -100, -20), 0);
  assert.equal(normalizeDb(-20, -100, -20), 1);
  assert.equal(normalizeDb(-60, -100, -20), 0.5);
  assert.equal(normalizeDb(-200, -100, -20), 0); // below range
  assert.equal(normalizeDb(0, -100, -20), 1); // above range
});

test("every palette maps 0 to black and 1 to white", () => {
  for (const p of [hotPalette, grayscalePalette, icePalette]) {
    assert.deepEqual(p(0), [0, 0, 0]);
    assert.deepEqual(p(1), [255, 255, 255]);
  }
  // Distinct mid-tones: hot is reddish, ice is bluish, grayscale is neutral.
  assert.ok(hotPalette(0.33)[0] > hotPalette(0.33)[2]);
  assert.ok(icePalette(0.33)[2] > icePalette(0.33)[0]);
  const [gr, gg, gb] = grayscalePalette(0.5);
  assert.ok(gr === gg && gg === gb);
});

test("PALETTE_NAMES lists the selectable palettes", () => {
  assert.deepEqual([...PALETTE_NAMES].sort(), ["grayscale", "hot", "ice"]);
});

test("rowRgba yields one opaque RGBA pixel per bin using the given palette", () => {
  const row = rowRgba([-100, -20], -100, -20, hotPalette);
  assert.equal(row.length, 8);
  assert.deepEqual([...row.slice(0, 4)], [0, 0, 0, 255]); // min -> black
  assert.deepEqual([...row.slice(4, 8)], [255, 255, 255, 255]); // max -> white
});
