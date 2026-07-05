// Typed wrappers over the rig control REST API (ARC-10 → ARC-04). Kept separate
// from the generic ApiClient so each rig operation has a small, testable, typed
// surface. Frequency (A21, FR-RIG-01/02); mode / PTT / S-meter follow in
// A22–A24.

import type { ApiClient } from "./api.ts";

/** The allowlisted operating modes, matching the backend rigctld tokens. */
export type RigMode =
  | "USB"
  | "LSB"
  | "CW"
  | "CWR"
  | "AM"
  | "FM"
  | "WFM"
  | "RTTY"
  | "PKTUSB"
  | "PKTLSB";

export const RIG_MODES: readonly RigMode[] = [
  "USB",
  "LSB",
  "CW",
  "CWR",
  "AM",
  "FM",
  "WFM",
  "RTTY",
  "PKTUSB",
  "PKTLSB",
];

interface FrequencyResponse {
  readonly hz: number;
}

interface ModeResponse {
  readonly mode: RigMode;
}

interface SmeterResponse {
  readonly strength: number;
}

/** Read the current rig frequency in Hz (FR-RIG-01). */
export async function getFrequency(api: ApiClient, accessToken: string): Promise<number> {
  const response = await api.get<FrequencyResponse>("/api/rig/frequency", accessToken);
  return response.hz;
}

/**
 * Set the rig frequency in Hz (FR-RIG-02). Out-of-range values are rejected
 * server-side (the backend returns 400 → [`ApiError`]).
 */
export async function setFrequency(
  api: ApiClient,
  accessToken: string,
  hz: number,
): Promise<void> {
  await api.post("/api/rig/frequency", accessToken, { hz });
}

/** Read the current operating mode (FR-RIG-03). */
export async function getMode(api: ApiClient, accessToken: string): Promise<RigMode> {
  const response = await api.get<ModeResponse>("/api/rig/mode", accessToken);
  return response.mode;
}

/** Set the operating mode, with an optional passband in Hz (FR-RIG-04). */
export async function setMode(
  api: ApiClient,
  accessToken: string,
  mode: RigMode,
  passbandHz = 0,
): Promise<void> {
  await api.post("/api/rig/mode", accessToken, { mode, passband_hz: passbandHz });
}

/** Activate or deactivate PTT (FR-RIG-05). Requires the Operator role. */
export async function setPtt(
  api: ApiClient,
  accessToken: string,
  transmit: boolean,
): Promise<void> {
  await api.post("/api/rig/ptt", accessToken, { transmit });
}

/** Read the S-meter strength (FR-RIG-06). */
export async function getSmeter(api: ApiClient, accessToken: string): Promise<number> {
  const response = await api.get<SmeterResponse>("/api/rig/smeter", accessToken);
  return response.strength;
}
