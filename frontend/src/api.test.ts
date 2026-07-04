import { test } from "node:test";
import assert from "node:assert/strict";
import { ApiClient, ApiError, type FetchLike } from "./api.ts";

function jsonResponse(body: unknown, status = 200): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { "content-type": "application/json" },
  });
}

test("login parses tokens and computes the expiry", async () => {
  const fetchMock: FetchLike = async () =>
    jsonResponse({
      access_token: "a",
      refresh_token: "r",
      expires_in: 900,
      role: "operator",
    });
  const client = new ApiClient({ baseUrl: "http://x/", fetch: fetchMock, now: () => 1_000 });

  const tokens = await client.login("op", "pw");
  assert.equal(tokens.accessToken, "a");
  assert.equal(tokens.refreshToken, "r");
  assert.equal(tokens.role, "operator");
  // now (1_000 ms) + expires_in (900 s) -> 901_000 ms
  assert.equal(tokens.expiresAt, 901_000);
});

test("login rejects with ApiError on 401", async () => {
  const fetchMock: FetchLike = async () => new Response("unauthorized", { status: 401 });
  const client = new ApiClient({ baseUrl: "http://x", fetch: fetchMock, now: () => 0 });

  await assert.rejects(
    () => client.login("op", "wrong"),
    (error: unknown) => error instanceof ApiError && error.status === 401,
  );
});

test("get sends the bearer token", async () => {
  let seenAuth: string | null = null;
  const fetchMock: FetchLike = async (_input, init) => {
    const headers = new Headers(init?.headers);
    seenAuth = headers.get("authorization");
    return jsonResponse({ hz: 14_074_000 });
  };
  const client = new ApiClient({ baseUrl: "http://x", fetch: fetchMock, now: () => 0 });

  const result = await client.get<{ hz: number }>("/api/rig/frequency", "tok");
  assert.equal(result.hz, 14_074_000);
  assert.equal(seenAuth, "Bearer tok");
});
