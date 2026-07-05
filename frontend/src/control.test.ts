import { test } from "node:test";
import assert from "node:assert/strict";
import { ApiClient, type FetchLike } from "./api.ts";
import { getFrequency, setFrequency } from "./control.ts";

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
