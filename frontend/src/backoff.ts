// Exponential-backoff helper for WebSocket reconnection (NFR-REL-01).
// Pure and framework-free so it is unit-testable without a browser.

export interface BackoffOptions {
  /** Delay for the first retry (attempt 0), in milliseconds. */
  readonly baseMs: number;
  /** Hard cap on any single delay, in milliseconds (NFR-REL-01: 30 000). */
  readonly maxMs: number;
}

/**
 * The delay before reconnect attempt `attempt` (0-based): `base * 2^attempt`,
 * capped at `maxMs`. NFR-REL-01 requires the cap to be at most 30 s.
 */
export function backoffDelay(attempt: number, options: BackoffOptions): number {
  if (attempt < 0) {
    return options.baseMs;
  }
  const exponential = options.baseMs * 2 ** attempt;
  return Math.min(exponential, options.maxMs);
}

/** The default reconnection schedule: 1 s base, 30 s cap (NFR-REL-01). */
export const DEFAULT_BACKOFF: BackoffOptions = { baseMs: 1_000, maxMs: 30_000 };
