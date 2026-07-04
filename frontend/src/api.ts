// Backend API client (ARC-10). A thin, typed wrapper over `fetch` for the auth
// endpoints (FR-AUTH-01..05) and the authenticated control API. `fetch` and the
// clock are injected so the client is unit-testable without a browser or server.

import type { Role, Tokens } from "./session.ts";

interface LoginResponse {
  readonly access_token: string;
  readonly refresh_token: string;
  readonly expires_in: number;
  readonly role: Role;
}

export type FetchLike = (input: string, init?: RequestInit) => Promise<Response>;

export interface ApiClientOptions {
  readonly baseUrl: string;
  readonly fetch: FetchLike;
  /** Returns the current time in epoch milliseconds. */
  readonly now: () => number;
}

/** An error carrying the HTTP status of a failed request. */
export class ApiError extends Error {
  readonly status: number;

  constructor(status: number, message: string) {
    super(message);
    this.name = "ApiError";
    this.status = status;
  }
}

export class ApiClient {
  private readonly baseUrl: string;
  private readonly fetchImpl: FetchLike;
  private readonly now: () => number;

  constructor(options: ApiClientOptions) {
    this.baseUrl = options.baseUrl.replace(/\/+$/, "");
    this.fetchImpl = options.fetch;
    this.now = options.now;
  }

  /** Authenticate with a name and password, returning session tokens. */
  async login(name: string, password: string): Promise<Tokens> {
    const response = await this.fetchImpl(`${this.baseUrl}/auth/login`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ name, password }),
    });
    if (!response.ok) {
      throw new ApiError(response.status, "login failed");
    }
    return this.toTokens((await response.json()) as LoginResponse);
  }

  /** Exchange a refresh token for a new token pair (FR-AUTH-03). */
  async refresh(refreshToken: string): Promise<Tokens> {
    const response = await this.fetchImpl(`${this.baseUrl}/auth/refresh`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ refresh_token: refreshToken }),
    });
    if (!response.ok) {
      throw new ApiError(response.status, "refresh failed");
    }
    return this.toTokens((await response.json()) as LoginResponse);
  }

  /** Invalidate the session server-side (FR-AUTH-05). Best-effort. */
  async logout(accessToken: string, refreshToken: string): Promise<void> {
    await this.fetchImpl(`${this.baseUrl}/auth/logout`, {
      method: "POST",
      headers: {
        "content-type": "application/json",
        authorization: `Bearer ${accessToken}`,
      },
      body: JSON.stringify({ refresh_token: refreshToken }),
    });
  }

  /** Authenticated GET returning parsed JSON of type `T`. */
  async get<T>(path: string, accessToken: string): Promise<T> {
    const response = await this.fetchImpl(`${this.baseUrl}${path}`, {
      headers: { authorization: `Bearer ${accessToken}` },
    });
    if (!response.ok) {
      throw new ApiError(response.status, `GET ${path} failed`);
    }
    return (await response.json()) as T;
  }

  private toTokens(data: LoginResponse): Tokens {
    return {
      accessToken: data.access_token,
      refreshToken: data.refresh_token,
      role: data.role,
      expiresAt: this.now() + data.expires_in * 1_000,
    };
  }
}
