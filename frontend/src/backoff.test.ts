import { test } from "node:test";
import assert from "node:assert/strict";
import { backoffDelay, DEFAULT_BACKOFF } from "./backoff.ts";

test("first attempt uses the base delay", () => {
  assert.equal(backoffDelay(0, DEFAULT_BACKOFF), 1_000);
});

test("delay doubles each attempt", () => {
  assert.equal(backoffDelay(1, DEFAULT_BACKOFF), 2_000);
  assert.equal(backoffDelay(2, DEFAULT_BACKOFF), 4_000);
  assert.equal(backoffDelay(3, DEFAULT_BACKOFF), 8_000);
});

test("delay is capped at maxMs (NFR-REL-01: 30 s)", () => {
  assert.equal(backoffDelay(20, DEFAULT_BACKOFF), 30_000);
  for (let attempt = 0; attempt < 30; attempt++) {
    assert.ok(backoffDelay(attempt, DEFAULT_BACKOFF) <= 30_000);
  }
});
