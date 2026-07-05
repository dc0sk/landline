import { test } from "node:test";
import assert from "node:assert/strict";
import { ApiClient, type FetchLike } from "./api.ts";
import {
  getFrequency,
  getMode,
  getSmeter,
  setFrequency,
  setMode,
  setPtt,
} from "./control.ts";

/** A fetch mock that records the last request and returns `body` as JSON. */
function recorder(body: unknown, status = 200) {
  const seen: { method?: string; auth: string | null; body: string | null } = {
    auth: null,
    body: null,
  };
  const fetchMock: FetchLike = async (_input, init) => {
    seen.method = init?.method ?? "GET";
    seen.auth = new Headers(init?.headers).get("authorization");
    seen.body = (init?.body as string | undefined) ?? null;
    const responseInit: ResponseInit = { status };
    if (body !== null) {
      responseInit.headers = { "content-type": "application/json" };
    }
    return new Response(body === null ? null : JSON.stringify(body), responseInit);
  };
  return { seen, fetchMock };
}

test("getFrequency reads the hz field", async () => {
  const fetchMock: FetchLike = async () =>
    new Response(JSON.stringify({ hz: 14_074_000 }), {
      status: 200,
      headers: { "content-type": "application/json" },
    });
  const api = new ApiClient({ baseUrl: "http://x", fetch: fetchMock, now: () => 0 });
  assert.equal(await getFrequency(api, "tok"), 14_074_000);
});

test("setFrequency POSTs the hz body with a bearer token", async () => {
  let method: string | undefined;
  let auth: string | null = null;
  let body: string | null = null;
  const fetchMock: FetchLike = async (_input, init) => {
    method = init?.method;
    auth = new Headers(init?.headers).get("authorization");
    body = init?.body as string;
    return new Response(null, { status: 204 });
  };
  const api = new ApiClient({ baseUrl: "http://x", fetch: fetchMock, now: () => 0 });

  await setFrequency(api, "tok", 14_100_000);
  assert.equal(method, "POST");
  assert.equal(auth, "Bearer tok");
  assert.deepEqual(JSON.parse(body ?? "null"), { hz: 14_100_000 });
});

test("getMode reads the mode token", async () => {
  const { fetchMock } = recorder({ mode: "USB" });
  const api = new ApiClient({ baseUrl: "http://x", fetch: fetchMock, now: () => 0 });
  assert.equal(await getMode(api, "tok"), "USB");
});

test("setMode POSTs mode and passband", async () => {
  const { seen, fetchMock } = recorder(null, 204);
  const api = new ApiClient({ baseUrl: "http://x", fetch: fetchMock, now: () => 0 });
  await setMode(api, "tok", "LSB", 2_400);
  assert.equal(seen.method, "POST");
  assert.deepEqual(JSON.parse(seen.body ?? "null"), { mode: "LSB", passband_hz: 2_400 });
});

test("setPtt POSTs the transmit flag", async () => {
  const { seen, fetchMock } = recorder(null, 204);
  const api = new ApiClient({ baseUrl: "http://x", fetch: fetchMock, now: () => 0 });
  await setPtt(api, "tok", true);
  assert.deepEqual(JSON.parse(seen.body ?? "null"), { transmit: true });
});

test("getSmeter reads the strength", async () => {
  const { fetchMock } = recorder({ strength: -54 });
  const api = new ApiClient({ baseUrl: "http://x", fetch: fetchMock, now: () => 0 });
  assert.equal(await getSmeter(api, "tok"), -54);
});
