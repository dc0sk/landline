// Frontend entry point (ARC-10): authenticated session bootstrap (A19).
// Wires the login form to the API client and toggles between the login and
// application views based on session state. DOM glue only — the testable logic
// lives in api.ts / session.ts / backoff.ts.

import { ApiClient } from "./api.ts";
import { loadAudioDevices, type AudioDevice } from "./audio-devices.ts";
import {
  getFrequency,
  getMode,
  getSmeter,
  RIG_MODES,
  setFrequency,
  setMode,
  setPtt,
  type RigMode,
} from "./control.ts";
import { AudioPlayer, MicCapture } from "./audio-player.ts";
import { Session } from "./session.ts";
import { browserSocket } from "./socket.ts";
import { TelemetryClient } from "./telemetry-client.ts";
import { PALETTES, PALETTE_NAMES, WaterfallRenderer } from "./waterfall.ts";

const BASE_URL = (globalThis as { LANDLINE_API_BASE?: string }).LANDLINE_API_BASE ?? "";

const api = new ApiClient({
  baseUrl: BASE_URL,
  fetch: (input, init) => fetch(input, init),
  now: () => Date.now(),
});
const session = new Session();

let pttActive = false;
let telemetryClient: TelemetryClient | null = null;
let audioPlayer: AudioPlayer | null = null;
let micCapture: MicCapture | null = null;
let waterfallRenderer: WaterfallRenderer | null = null;

function wsUrl(): string {
  if (BASE_URL) {
    return `${BASE_URL.replace(/^http/, "ws")}/ws`;
  }
  const scheme = window.location.protocol === "https:" ? "wss:" : "ws:";
  return `${scheme}//${window.location.host}/ws`;
}

function startTelemetry(): void {
  const tokens = session.current;
  if (tokens === null || telemetryClient !== null) {
    return;
  }
  const context = byId<HTMLCanvasElement>("waterfall").getContext("2d");
  if (context === null) {
    return;
  }
  waterfallRenderer = new WaterfallRenderer(context, { minDb: -90, maxDb: -10 });
  audioPlayer = new AudioPlayer(48_000);
  audioPlayer.start();
  telemetryClient = new TelemetryClient({
    url: wsUrl(),
    token: tokens.accessToken,
    connect: browserSocket,
    onFrame: (frame) => waterfallRenderer?.push(frame.bins),
    onAudio: (frame) => audioPlayer?.push(frame),
    onError: (message) => showRigError(message),
  });
  telemetryClient.start();
}

function stopTelemetry(): void {
  stopMicTx();
  telemetryClient?.stop();
  telemetryClient = null;
  audioPlayer?.stop();
  audioPlayer = null;
  waterfallRenderer = null;
}

function fillDeviceSelect(select: HTMLSelectElement, devices: AudioDevice[]): void {
  select.replaceChildren();
  for (const device of devices) {
    const option = document.createElement("option");
    option.value = device.deviceId;
    option.textContent = device.label;
    select.append(option);
  }
}

async function refreshAudioDevices(): Promise<void> {
  const media = navigator.mediaDevices;
  if (typeof media?.enumerateDevices !== "function") {
    byId("audio-note").textContent = "Audio device selection is unavailable in this browser.";
    return;
  }
  // Labels are hidden until microphone permission is granted; requesting it
  // once unlocks them (FR-AUD-03/04, NFR-COMPAT-07).
  try {
    const stream = await media.getUserMedia({ audio: true });
    for (const track of stream.getTracks()) {
      track.stop();
    }
  } catch {
    byId("audio-note").textContent = "Microphone permission denied; device names may be hidden.";
  }
  const devices = await loadAudioDevices(media);
  fillDeviceSelect(byId<HTMLSelectElement>("audio-input"), devices.inputs);
  fillDeviceSelect(byId<HTMLSelectElement>("audio-output"), devices.outputs);
}

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
    setPttUi(false);
    void refreshFrequency();
    void refreshMode();
    void refreshSmeter();
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

async function refreshMode(): Promise<void> {
  const tokens = session.current;
  if (tokens === null) {
    return;
  }
  try {
    byId<HTMLSelectElement>("mode-select").value = await getMode(api, tokens.accessToken);
  } catch {
    // Leave the selector as-is if the rig is unreachable.
  }
}

async function handleModeChange(): Promise<void> {
  const tokens = session.current;
  if (tokens === null) {
    return;
  }
  showRigError("");
  const mode = byId<HTMLSelectElement>("mode-select").value as RigMode;
  try {
    await setMode(api, tokens.accessToken, mode);
  } catch {
    showRigError("Could not set mode.");
  }
}

function setPttUi(active: boolean): void {
  pttActive = active;
  const button = byId<HTMLButtonElement>("ptt-button");
  button.textContent = active ? "ON AIR — release" : "PTT (transmit)";
  button.classList.toggle("transmitting", active);
  button.setAttribute("aria-pressed", String(active));
}

async function handlePtt(): Promise<void> {
  const tokens = session.current;
  if (tokens === null) {
    return;
  }
  showRigError("");
  const next = !pttActive;
  try {
    await setPtt(api, tokens.accessToken, next);
    setPttUi(next);
    // Mic TX is gated on PTT (FR-AUD-02): capture only while transmitting.
    if (next) {
      await startMicTx();
    } else {
      stopMicTx();
    }
  } catch {
    showRigError("PTT not permitted (Operator role required) or rig unavailable.");
  }
}

async function startMicTx(): Promise<void> {
  if (micCapture !== null) {
    return;
  }
  const capture = new MicCapture();
  const deviceId = byId<HTMLSelectElement>("audio-input").value || undefined;
  try {
    await capture.start(deviceId, (frame) => telemetryClient?.sendAudio(frame));
    micCapture = capture;
  } catch {
    showRigError("Microphone unavailable for transmit.");
  }
}

function stopMicTx(): void {
  micCapture?.stop();
  micCapture = null;
}

async function refreshSmeter(): Promise<void> {
  const tokens = session.current;
  if (tokens === null) {
    return;
  }
  try {
    const strength = await getSmeter(api, tokens.accessToken);
    byId("smeter-display").textContent = `${strength} dB`;
  } catch {
    byId("smeter-display").textContent = "—";
  }
}

async function handleLogin(event: SubmitEvent): Promise<void> {
  event.preventDefault();
  showError("");
  const name = byId<HTMLInputElement>("name").value;
  const password = byId<HTMLInputElement>("password").value;
  try {
    session.set(await api.login(name, password));
    startTelemetry();
    void refreshAudioDevices();
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
  stopTelemetry();
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

  const modeSelect = byId<HTMLSelectElement>("mode-select");
  for (const mode of RIG_MODES) {
    const option = document.createElement("option");
    option.value = mode;
    option.textContent = mode;
    modeSelect.append(option);
  }
  modeSelect.addEventListener("change", () => {
    void handleModeChange();
  });
  byId<HTMLButtonElement>("ptt-button").addEventListener("click", () => {
    void handlePtt();
  });

  const paletteSelect = byId<HTMLSelectElement>("palette-select");
  for (const name of PALETTE_NAMES) {
    const option = document.createElement("option");
    option.value = name;
    option.textContent = name;
    paletteSelect.append(option);
  }
  paletteSelect.addEventListener("change", () => {
    const palette = PALETTES[paletteSelect.value];
    if (palette !== undefined) {
      waterfallRenderer?.setPalette(palette);
    }
  });

  window.setInterval(() => {
    void maybeRefresh();
  }, 30_000);
  // Poll the S-meter while signed in (FR-RIG-06 point-read; streaming is Phase 2).
  window.setInterval(() => {
    if (session.isAuthenticated(Date.now())) {
      void refreshSmeter();
    }
  }, 1_000);
  render();
}

main();
