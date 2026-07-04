// Client-side session state (ARC-10). Realises the client half of the auth
// requirements: hold the tokens issued by the backend (FR-AUTH-01..05) and know
// when the access token has expired or is about to (FR-AUTH-02/03/05).
//
// Security note: tokens are held only in memory — never in localStorage — so an
// XSS foothold cannot exfiltrate a persisted token, and a page reload requires a
// fresh login. This is a deliberate trade-off aligned with the security-first
// governance charter.

export type Role = "admin" | "operator" | "observer";

export interface Tokens {
  readonly accessToken: string;
  readonly refreshToken: string;
  readonly role: Role;
  /** Epoch milliseconds at which the access token expires. */
  readonly expiresAt: number;
}

export class Session {
  private tokens: Tokens | null = null;

  set(tokens: Tokens): void {
    this.tokens = tokens;
  }

  clear(): void {
    this.tokens = null;
  }

  get current(): Tokens | null {
    return this.tokens;
  }

  /** Whether there is a non-expired access token at time `now` (epoch ms). */
  isAuthenticated(now: number): boolean {
    return this.tokens !== null && now < this.tokens.expiresAt;
  }

  /**
   * Whether the access token is within `windowMs` of expiry and should be
   * proactively refreshed (FR-AUTH-03).
   */
  needsRefresh(now: number, windowMs: number): boolean {
    return this.tokens !== null && now >= this.tokens.expiresAt - windowMs;
  }

  /** Whether the current role satisfies a required minimum (FR-AUTH-04). */
  hasRole(required: Role): boolean {
    if (this.tokens === null) {
      return false;
    }
    return RANK[this.tokens.role] >= RANK[required];
  }
}

const RANK: Record<Role, number> = { admin: 3, operator: 2, observer: 1 };
