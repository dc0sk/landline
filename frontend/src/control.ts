// Typed wrappers over the rig control REST API (ARC-10 → ARC-04). Kept separate
// from the generic ApiClient so each rig operation has a small, testable, typed
// surface. Frequency (A21, FR-RIG-01/02); mode / PTT / S-meter follow in
// A22–A24.

import type { ApiClient } from "./api.ts";

interface FrequencyResponse {
  readonly hz: number;
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
