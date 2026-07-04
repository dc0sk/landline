import { test } from "node:test";
import assert from "node:assert/strict";
import { Session, type Tokens } from "./session.ts";

function tokens(overrides: Partial<Tokens> = {}): Tokens {
  return {
    accessToken: "access",
    refreshToken: "refresh",
    role: "operator",
    expiresAt: 10_000,
    ...overrides,
  };
}

test("a fresh session is not authenticated", () => {
  const session = new Session();
  assert.equal(session.isAuthenticated(0), false);
  assert.equal(session.current, null);
});

test("authenticated until the access token expires", () => {
  const session = new Session();
  session.set(tokens({ expiresAt: 10_000 }));
  assert.equal(session.isAuthenticated(9_999), true);
  assert.equal(session.isAuthenticated(10_000), false);
});

test("needsRefresh inside the refresh window", () => {
  const session = new Session();
  session.set(tokens({ expiresAt: 10_000 }));
  assert.equal(session.needsRefresh(8_999, 1_000), false);
  assert.equal(session.needsRefresh(9_000, 1_000), true);
});

test("clear drops the tokens", () => {
  const session = new Session();
  session.set(tokens());
  session.clear();
  assert.equal(session.current, null);
  assert.equal(session.isAuthenticated(0), false);
});

test("hasRole enforces the privilege ordering", () => {
  const session = new Session();
  session.set(tokens({ role: "operator" }));
  assert.equal(session.hasRole("observer"), true);
  assert.equal(session.hasRole("operator"), true);
  assert.equal(session.hasRole("admin"), false);
});
