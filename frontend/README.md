# landline frontend

Browser-native **TypeScript** frontend (ARC-10, ADR-01) for landline. No
framework, no bundler: erasable TypeScript compiled with `tsc`, served as static
files. Talks to the backend over the authenticated REST API (and, from Phase 2,
the WebSocket telemetry channel).

## Develop

```sh
npm install
npm run typecheck   # tsc --noEmit (strict)
npm test            # node --test with type stripping (unit tests)
npm run build       # tsc -> dist/
```

Then serve the directory (any static server) and open `index.html`. Point the
client at a non-same-origin backend by setting `window.LANDLINE_API_BASE` before
`main.js` loads (e.g. for split-host deployments).

## Layout

| File | Role |
|---|---|
| `src/session.ts` | In-memory token/session state (FR-AUTH-01..05); tokens are never persisted to storage |
| `src/api.ts` | Typed `fetch` client for the auth + control API |
| `src/backoff.ts` | Exponential-backoff schedule for WS reconnect (NFR-REL-01) |
| `src/main.ts` | DOM bootstrap: login/logout wiring and view toggling |

Security note: access/refresh tokens live only in memory, so an XSS foothold
cannot read a persisted token and a reload requires re-login — a deliberate
trade-off per the security-first governance charter.
