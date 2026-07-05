// Frontend entry point (ARC-10): authenticated session bootstrap (A19).
// Wires the login form to the API client and toggles between the login and
// application views based on session state. DOM glue only — the testable logic
// lives in api.ts / session.ts / backoff.ts.

import { ApiClient } from "./api.ts";
import { getFrequency, setFrequency } from "./control.ts";
import { Session } from "./session.ts";

const BASE_URL = (globalThis as { LANDLINE_API_BASE?: string }).LANDLINE_API_BASE ?? "";

const api = new ApiClient({
  baseUrl: BASE_URL,
  fetch: (input, init) => fetch(input, init),
  now: () => Date.now(),
});
const session = new Session();

function byId<T extends HTMLElement = HTMLElement>(id: string): T {
  const element = document.getElementById(id);
  if (element === null) {
    throw new Error(`missing element #${id}`);
  }
  return element as T;
}

function showError(message: string): void {
  byId("error").textContent = message;
}

function render(): void {
  const authenticated = session.isAuthenticated(Date.now());
  byId("login-view").hidden = authenticated;
  byId("app-view").hidden = !authenticated;
  const tokens = session.current;
  if (tokens !== null) {
    byId("who").textContent = `Signed in as ${tokens.role}`;
    void refreshFrequency();
  }
}

function showRigError(message: string): void {
  byId("rig-error").textContent = message;
}

async function refreshFrequency(): Promise<void> {
  const tokens = session.current;
  if (tokens === null) {
    return;
  }
  try {
    const hz = await getFrequency(api, tokens.accessToken);
    byId("freq-display").textContent = hz.toLocaleString();
  } catch {
    byId("freq-display").textContent = "unavailable";
  }
}

async function handleSetFrequency(event: SubmitEvent): Promise<void> {
  event.preventDefault();
  showRigError("");
  const tokens = session.current;
  if (tokens === null) {
    return;
  }
  const hz = Number.parseInt(byId<HTMLInputElement>("freq-input").value, 10);
  if (!Number.isFinite(hz)) {
    showRigError("Enter a frequency in Hz.");
    return;
  }
  try {
    await setFrequency(api, tokens.accessToken, hz);
    await refreshFrequency();
  } catch {
    showRigError("Could not set frequency (out of range or rig unavailable).");
  }
}

async function handleLogin(event: SubmitEvent): Promise<void> {
  event.preventDefault();
  showError("");
  const name = byId<HTMLInputElement>("name").value;
  const password = byId<HTMLInputElement>("password").value;
  try {
    session.set(await api.login(name, password));
    render();
  } catch {
    // Generic message: the server does not reveal why auth failed, and neither
    // do we (no user enumeration).
    showError("Login failed. Check your credentials.");
  }
}

async function handleLogout(): Promise<void> {
  const tokens = session.current;
  if (tokens !== null) {
    try {
      await api.logout(tokens.accessToken, tokens.refreshToken);
    } catch {
      // Best-effort: clear the local session regardless.
    }
  }
  session.clear();
  render();
}

async function maybeRefresh(): Promise<void> {
  const tokens = session.current;
  if (tokens !== null && session.needsRefresh(Date.now(), 60_000)) {
    try {
      session.set(await api.refresh(tokens.refreshToken));
    } catch {
      session.clear();
      render();
    }
  }
}

function main(): void {
  byId<HTMLFormElement>("login-form").addEventListener("submit", (event) => {
    void handleLogin(event);
  });
  byId<HTMLButtonElement>("logout").addEventListener("click", () => {
    void handleLogout();
  });
  byId<HTMLFormElement>("freq-form").addEventListener("submit", (event) => {
    void handleSetFrequency(event);
  });
  window.setInterval(() => {
    void maybeRefresh();
  }, 30_000);
  render();
}

main();
