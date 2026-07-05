---
title: Action List
status: Draft
version: "0.27"
updated: 2026-07-05
authors:
  - Simon Keimer (DC0SK)
owns: []
---

# Action List

This is the ordered, working checklist that turns the backlog and roadmap into concrete next
steps. It is the **operational companion** to [backlog.md](backlog.md) and
[roadmap.md](roadmap.md): every action cites the backlog item(s) and requirement/test IDs it
advances — this document **references** those IDs, it does not define new ones. Actions carry
stable labels (A1, A2, …) so they can be referenced in commits and reviews.

License notice: This project is licensed under AGPL-3.0-only. See the top-level LICENSE file.

---

## 1. Current state

- **Phase 0 (RE baseline): complete** — all governance, requirements, security, test, and
  planning docs exist; Phase 0 exit criteria are ticked in [roadmap.md](roadmap.md) §4.
- **Trace gate: green** — 78 requirements declared, all 76 Must/Should covered by tests
  (`python3 scripts/trace-gate.py`). FR-RIG-07 and FR-SPEC-04 are Could-priority and
  intentionally uncovered per rule R3.
- **Phase 1 — Secure Control MVP: in progress.** Backend skeleton (A1–A5) plus **auth &
  RBAC (A6)** have landed: Cargo workspace (`backend` + `xtask`), Axum/Tokio/Tower server,
  structured tracing, single-file TOML config, activated CI + git hooks, verified aarch64
  cross-build, and the ARC-02 auth module (HS256 JWT + argon2 login + refresh rotation +
  logout revocation + RBAC extractor) and the ARC-03 security middleware (A7/A8: per-client
  rate limiting, body-size limit, CORS allowlist). Transport decision recorded in ADR-08
  (TLS/WSS + JWT; TLS-PSK rejected). The ARC-07 audit log (A9: hash-chained,
  tamper-evident, Admin-viewable) is in place, the ARC-04 rigctld adapter (A10) has landed,
  the **rig control endpoints** (A11–A14) are live, and the **Phase 1 backend is complete**
  (A15 S-meter read, A16 circuit breaker, A17 GPIO). The **frontend has started** (A18–A19:
  TypeScript project + authenticated session bootstrap, verified with typecheck + 11 tests +
  build), A20–A21 (WS client + frequency UI), and A22–A24 (mode selector, PTT button,
  S-meter display) have landed (22 frontend tests). The full rig-control UI is now wired
  over the REST API, with responsive CSS (A25) and a hardened systemd unit (A26).
  **Phase 1 (A1–A27) is development-complete.** The A27 exit review recorded the honest
  position: software-complete and green under automation (55 Rust + 22 frontend tests), with
  the formal exit gate held pending hardware-in-the-loop validation (Pi + rigctld + browser
  matrix) and the Phase-4 TLS front end. **Phase 2 has started (A28):** the backend spectrum
  path — FFT pipeline + authenticated WebSocket transport streaming spectrum frames — is done
  and tested (62 Rust tests), which also lands the Phase-1-deferred WS security items
  (TC-AUTH-01 WS auth, TC-SEC-05 WS frame limit). A29 added the browser **Canvas 2D waterfall**
  (no WebGL) consuming that stream over the reconnecting socket. A30 added the audio device
  selector (MediaDevices) + touch refinements. **Phase 2 is complete** (A28–A30; 29 frontend +
  62 Rust tests). The **Phase 2 exit review** reconciled roadmap §6 (met / software-complete-
  pending-browser+HIL / not met) with a dated assessment and a test-strategy §6b execution
  record: software-complete, formal gate pending browser-matrix + Pi HIL. **Phase 3 has started
  (A31, in progress):** the ARC-05 audio software core — jitter buffer (loss concealment) +
  codec seam + config — is done and unit-tested, C-free cross-build preserved. The native/HIL
  parts (libopus, CPAL, WS audio transport, browser Web Audio) are the remainder. **A34
  (split-host) is done** and **A33 (container) artifacts are done** (hardened Dockerfile +
  compose + decision-record skeleton; device-passthrough/latency + the accept/defer decision are
  Pi HIL). **Security remainders + TLS front end closed (A36):** the 0600 config check (BL-081),
  global panic sanitisation (BL-032 / NFR-SEC-09), and the nginx TLS/WSS reverse-proxy config
  (BL-100/101) are done — **retiring the TLS deferral (NFR-SEC-01 / TC-SEC-01)** that every
  phase gate had been waiting on, and moving BL-021 to Done. 71 Rust tests. **Remaining
  software-buildable work:** the ops docs (A38: rollback, runbook, release checklist) and the
  secrets-rotation policy (A35 / BL-012); everything else (audio pipeline, soak/load, browser
  matrix) is hardware-in-the-loop.
- **All software-buildable work is complete**, including the **authenticated WS audio RX
  transport** (BL-075 Done, BL-071 In Progress — binary audio frames from the shared sample
  source, per-session auth, tested). What remains is exclusively **hardware-in-the-loop /
  browser-matrix**: the audio device ends (CPAL capture/playback, libopus, browser Web Audio,
  mic TX) + latency (A31 remainder, A32), soak/load (A37), container passthrough/decision
  (A33 remainder), and the browser/Pi test execution that fills the final trace records (BL-104).
- **Feature-complete in software (2026-07-05).** The full audio pipeline (RX + TX transport,
  jitter buffer, PCM + feature-gated Opus, browser playback + mic capture), the spectrum/
  waterfall with palette selection (FR-SPEC-04), passband tuning (FR-RIG-07), and all Phase-4
  software (TLS proxy, ops docs, rotation policy) are done and tested (≈75 Rust + 38 frontend
  tests, all gates green). The remaining Phase-0 rotation item (BL-012) is **also done**
  (security.md §8.2). **Everything still open is hardware-in-the-loop or browser-matrix** —
  ready for rig/Pi/device testing.

---

## 2. Milestone: Phase 1 kickoff — backend foundation

Ordered to respect the backlog dependency graph (workspace → auth → security middleware →
rigctld adapter → control handlers → GPIO).

- [x] A1. Initialize Rust workspace (Tokio + Axum + Tower), with `cargo fmt`/`clippy` clean baseline — BL-020 · NFR-MAINT-01 — *done: `backend` + `xtask` crates; walking-skeleton router (`/healthz`, `/version`) with graceful SIGINT/SIGTERM shutdown; integration tests (`backend/tests/api.rs`, NFR-MAINT-02)*
- [x] A2. Activate latent tooling: rename `.github/workflows/ci.yml.disabled` → `ci.yml`, enable the commented-out Rust steps in `.githooks/pre-commit`/`.githooks/pre-push` (fmt, clippy `-D warnings`, test, audit), and promote the trace gate into `cargo xtask` — BL-020 · NFR-MAINT-01 — *done: CI `rust` job live; hooks run fmt+clippy (pre-commit) and test+audit (pre-push); `cargo xtask trace-gate`/`ci` wrap the gate*
- [x] A3. Set up cross-compilation for `aarch64-unknown-linux-gnu` in CI from day one — BL-083 · NFR-DEPLOY-01 · TC-DEPLOY-01–TC-DEPLOY-02 — *done: `.cargo/config.toml` linker override + CI cross-build step; verified locally (aarch64 ELF produced)*
- [x] A4. Implement structured tracing/logging integration (no secrets in logs) — BL-032 · NFR-SEC-09, NFR-SEC-12 · TC-SEC-09–TC-SEC-10 — *done: `telemetry::init` (env-filter, no credential emission, NFR-SEC-12); error-response sanitisation (NFR-SEC-09) via typed sanitised errors + a global `catch_panic_layer` that returns a generic 500 with no panic message (tested)*
- [x] A5. Implement TOML config loader with secure defaults and 0600 permission checks — BL-081 · NFR-DEPLOY-04 — *done: single-file TOML, loopback-default bind (NFR-SEC-13), `$LANDLINE_CONFIG` override, and 0600 enforcement — `load` rejects a group/world-accessible config on Unix (NFR-SEC-03); unit + file tests*
- [x] A6. Implement auth middleware (JWT issue/verify, expiry, role claims, RBAC) — BL-021 · FR-AUTH-01–FR-AUTH-05, NFR-SEC-02 · TC-AUTH-01–TC-AUTH-05 — *done: ARC-02 `auth` module — HS256 JWT (pure-Rust hmac/sha2, per ADR-08), argon2 login, short-lived access + rotating refresh, logout revocation, `AuthUser` extractor + RBAC (`require`); 11 unit + 6 HTTP tests. NFR-SEC-01 (TLS/WSS) is reverse-proxy/Phase 4 (TC-SEC-01); TC-SEC-02 entropy covered by unit test*
- [x] A7. Implement rate limiting and request/WS-frame size limits — BL-022 · NFR-SEC-04–NFR-SEC-05 · TC-SEC-04–TC-SEC-05 — *done: ARC-03 `security` module — per-client token-bucket rate limiter (default 10/s, keyed on peer IP) + `RequestBodyLimitLayer` (default 64 KiB). WS-frame size cap (TC-SEC-05) lands with the WS endpoints (Phase 2/3); reverse-proxy X-Forwarded-For keying is a Phase-4 follow-up*
- [x] A8. Implement CORS origin allowlist policy — BL-023 · NFR-SEC-06 · TC-SEC-06 — *done: `security::cors_layer` from configured `allowed_origins` (empty = deny all cross-origin); GET/POST + Authorization/Content-Type headers*
- [x] A9. Implement audit log subsystem (append-only, state-changing actions + auth failures) — BL-024 · FR-AUDIT-01–FR-AUDIT-04 · TC-AUDIT-01–TC-AUDIT-02 — *done: ARC-07 `audit` module — SHA-256 hash-chained tamper-evident events (`verify_chain`), timestamp/IP/user/action/params (FR-AUDIT-02), durable append file, Admin-only `GET /api/audit`. Auth failures logged with IP, no password (FR-AUDIT-04/NFR-SEC-12). TC-AUDIT-01 (rig-action entry) verified once the rig handlers landed (A11–A13); FR-AUDIT-03 retention is deployment log-rotation*
- [x] A10. Implement rigctld TCP adapter with command allowlist/sanitisation — BL-025 · FR-RIG-08–FR-RIG-09 · TC-RIG-07–TC-RIG-08 — *done: ARC-04 `rig` module — typed async hamlib/rigctld TCP client (freq/mode/PTT/S-meter), allowlisted `Mode` enum + numeric range validation (FR-RIG-09/NFR-SEC-08, injection-proof by construction), async-mutex-serialised access (FR-RIG-10), reconnect-on-failure. Tested against a mock rigctld (TC-RIG-07) + validation units (TC-RIG-08/TC-SEC-08). HTTP 400/502 mapping ready for the A11+ handlers*
- [x] A11. Implement frequency read/set handlers — BL-026 · FR-RIG-01–FR-RIG-02 · TC-RIG-01–TC-RIG-02 — *done: `control` module — `GET/POST /api/rig/frequency` (Operator), out-of-range → 400, set audited*
- [x] A12. Implement mode read/set handlers — BL-027 · FR-RIG-03–FR-RIG-04 · TC-RIG-03 — *done: `GET/POST /api/rig/mode` (Operator), unsupported/injection mode → 400 (NFR-SEC-08), set audited*
- [x] A13. Implement PTT handler with role check and safety timeout — BL-028 · FR-RIG-05, NFR-SEC-07 · TC-RIG-04–TC-RIG-05, TC-SEC-07 — *done: `POST /api/rig/ptt` (Operator); `PttGuard` server-side safety timeout auto-unkeys (NFR-SEC-07/TC-SEC-07); Observer denied → 403 and audited (TC-RIG-05)*
- [x] A14. Implement rig access mutex for concurrent clients — BL-030 · FR-RIG-10 · TC-RIG-09 — *done in the adapter (A10): all rigctld commands serialise through an async mutex, giving exclusive access across concurrent clients*
- [x] A15. Implement S-meter streaming — BL-029 · FR-RIG-06 · TC-RIG-06 — *read path done: `GET /api/rig/smeter` (Observer+). Continuous streaming at a configured cadence rides the Phase-2 WS telemetry channel (ADR-02)*
- [x] A16. Implement rigctld reconnect/circuit-breaker — BL-031 · NFR-REL-02 · TC-REL-02 — *done: adapter reconnects on failure + `CircuitBreaker` (opens after N failures, fail-fast, half-open after cooldown → 503); unit-tested. TC-REL-02 kill/restart is a System test needing real rigctld*
- [x] A17. Implement GPIO control API (≥ 5 digital pins, allowlist, safe startup states, role-gated) — BL-033 · FR-GPIO-01, NFR-SEC-16 · TC-GPIO-01, TC-SEC-15 — *done: ARC-08 `gpio` module — pin allowlist, safe startup states (NFR-SEC-16), Operator-gated `GET/POST /api/gpio/{pin}`, audited, in-memory backend. Non-allowlisted → 403, input pins not drivable. Tested (TC-SEC-15). Real Pi sysfs/gpiod backend is a deploy-time adapter; TC-GPIO-01 is a hardware System test*

## 3. Milestone: Phase 1 — frontend and deployment

Frontend bootstrap can start in parallel once the auth contract (A6) is stable.

- [x] A18. Initialize TypeScript/HTML5 frontend project — BL-040 · NFR-COMPAT-01–NFR-COMPAT-02 — *done: ARC-10 `frontend/` — erasable TypeScript (strict tsc, no bundler), `npm` typecheck/test/build, `index.html` shell, CI frontend job*
- [x] A19. Implement authenticated session bootstrap (login, token storage, logout) — BL-041 · FR-AUTH-01–FR-AUTH-05 · TC-AUTH-01–TC-AUTH-05 — *done: `api.ts` (login/refresh/logout/authed GET), `session.ts` (in-memory tokens — never persisted, XSS-safe; expiry/refresh-window/role checks), `main.ts` login/logout wiring + auto-refresh; 11 unit tests. Browser E2E across the matrix (TC-COMPAT) is the A25/Phase-2 pass*
- [x] A20. Implement WebSocket client with reconnect/backoff — BL-046 · NFR-REL-01 · TC-REL-01 — *done: `socket.ts` `ReconnectingSocket` — exponential backoff (1 s base, 30 s cap), attempt reset on open, injected transport + scheduler; 5 unit tests (TC-REL-01 logic). Activates with the Phase-2 WS telemetry channel (ADR-02); `browserSocket` adapts the real WebSocket*
- [x] A21. Implement frequency display and tuning control — BL-042 · FR-RIG-01–FR-RIG-02, NFR-COMPAT-06 · TC-RIG-01–TC-RIG-02, TC-COMPAT-01–TC-COMPAT-02 — *done: `control.ts` getFrequency/setFrequency over the REST API + `api.post`; frequency panel in `index.html`/`main.ts` (display + set, error states); 2 unit tests. Browser-matrix E2E (TC-COMPAT) is the A25/manual pass*
- [x] A22. Implement mode selector — BL-043 · FR-RIG-03–FR-RIG-04 · TC-RIG-03 — *done: `control.ts` getMode/setMode + `<select>` populated from `RIG_MODES`, set-on-change, reads current mode on load*
- [x] A23. Implement PTT button with visual transmit indicator — BL-044 · FR-RIG-05 · TC-RIG-04 — *done: `control.ts` setPtt + toggle button with `transmitting` class + `aria-pressed`; Operator-only rejection surfaced as an error*
- [x] A24. Implement S-meter display — BL-045 · FR-RIG-06 · TC-RIG-06 — *done: `control.ts` getSmeter + `<output>` polled every 1 s while signed in (continuous streaming rides the Phase-2 WS channel)*
- [x] A25. Responsive CSS layout (desktop 3-column, mobile vertical stack) — BL-047 · NFR-COMPAT-03–NFR-COMPAT-06 · TC-COMPAT-04–TC-COMPAT-07 — *done: `styles.css` — mobile-first 1-col → 3-col grid at ≥700px, 44px touch targets (NFR-COMPAT-06), light/dark via `prefers-color-scheme`, PTT transmit colour. Browser-matrix E2E (TC-COMPAT-04–07) is manual/hardware*
- [x] A26. Write systemd service unit (start/stop/restart, resource limits) — BL-080 · NFR-DEPLOY-02 · TC-DEPLOY-03 — *done: `deploy/systemd/landline.service` — hardened (unprivileged, empty caps, read-only root, syscall allowlist, private state dir for the audit log, MemoryMax/TasksMax) + `deploy/README.md`. TC-DEPLOY-03 (start/stop on Pi) is a hardware System test*
- [x] A27. Phase 1 exit review: run all scoped TC-RIG/TC-GPIO/TC-AUTH/TC-SEC/TC-PERF-01/TC-DEPLOY gates and tick roadmap.md §5 exit criteria — roadmap §5 · docs updated per governance change control — *done: reconciled §5 exit criteria (met / software-complete-pending-HIL / not met) + dated exit assessment in roadmap.md; Phase 1 execution record in test-strategy.md §6a. **Position: software-complete; formal gate held pending HIL (Pi + rigctld + browser matrix) and the Phase-4 TLS front end.***

## 4. Milestone: Phase 2 — spectrum, waterfall, mobile

- [x] A28. FFT pipeline + spectrum WebSocket stream at configurable rate — BL-050, BL-051, BL-054 · FR-SPEC-01–FR-SPEC-02 · TC-SPEC-01–TC-SPEC-02 — *done: ARC-06 `spectrum` module (rustfft, Hann window, dB bins, `SampleSource` seam + synthetic source) + ARC-01 `ws` module — authenticated WebSocket (first-message JWT auth, TC-AUTH-01; frame-size caps, NFR-SEC-05/TC-SEC-05) streaming spectrum frames at the configured 1–10 Hz rate. 4 FFT unit + 3 WS integration tests (real server + client). Real audio source is Phase 3; the Canvas renderer is A29*
- [x] A29. Canvas 2D waterfall renderer; verify on iOS Safari (no WebGL) — BL-052, BL-053 · FR-SPEC-03 · TC-SPEC-03–TC-SPEC-04 — *done: ARC-12 `waterfall.ts` — scrolling Canvas 2D waterfall (no WebGL → iOS-Safari-safe), pure normalise/palette/row logic unit-tested; `spectrum-client.ts` wires the reconnecting socket through the auth→subscribe handshake to the renderer. 5 unit tests. On-device iOS-Safari confirmation (TC-SPEC-04) is a browser-matrix run (BL-053)*
- [x] A30. Full browser matrix validation, touch optimisation, audio device selector UI — BL-060–BL-062 · NFR-COMPAT-01–NFR-COMPAT-07, FR-AUD-03–FR-AUD-04 · TC-COMPAT-01–TC-COMPAT-07, TC-AUD-03–TC-AUD-04 — *done (software): `audio-devices.ts` — MediaDevices enumeration (NFR-COMPAT-07) partitioned into input/output selectors with permission-unlock + labelled fallbacks (FR-AUD-03/04); touch refinements (16px font vs iOS zoom, `touch-action: manipulation`, full-width mobile controls, NFR-COMPAT-06). 2 unit tests. Full browser-matrix validation (TC-COMPAT-*, TC-AUD-03/04 on real devices) is the manual/HIL remainder (BL-060)*

## 5. Milestone: Phase 3 — audio, container evaluation, split-host

- [ ] A31. Bidirectional Opus audio pipeline over WSS with per-session auth — BL-070–BL-077 · FR-AUD-01–FR-AUD-06, NFR-SEC-01 · TC-AUD-01–TC-AUD-06 — *in progress — software done: ARC-05 `audio` module (`JitterBuffer` reorder + loss concealment FR-AUD-06; `Codec` seam + `PcmCodec` + `f32_to_pcm16`, FR-AUD-05) **and the authenticated WS binary audio RX transport** (subscribe `audio` → seq'd binary frames from the shared sample source, per-session auth BL-075; tested in `backend/tests/ws.rs`). 8 tests; aarch64 cross-build stays C-free. Frontend RX audio client added: `TelemetryClient` multiplexes spectrum (text) + audio (binary) on one socket (ADR-02); `audio-player.ts` parses binary frames → TS `JitterBuffer` → Web Audio (BL-072 In Progress; parse + jitter unit-tested). **Mic TX** (BL-073 In Progress): backend accepts Operator-only binary TX frames → `AudioSink` seam; frontend `MicCapture` (PTT-gated) encodes + sends. `split_frame`/`encodeAudioFrame`/`floatToPcm16` unit-tested. **Opus codec (BL-076 Done):** a `libopus`-backed `OpusCodec` (configurable bitrate, FR-AUD-05) behind the backend `opus` feature — off by default (C-free cross-build), enabled on the Pi with `--features opus`, round-trip-tested + CI-verified. Remaining is native/HIL: CPAL capture/playback on the Pi (BL-070/074), browser Opus + on-device mic/speaker, and end-to-end latency (BL-078/A32)*
- [ ] A32. Measure/document end-to-end audio latency on Pi 4 — BL-078 · NFR-PERF-02 · TC-PERF-02
- [ ] A33. Container evaluation (Dockerfile, compose, device passthrough, latency benchmark, decision record) — BL-090–BL-095 · NFR-DEPLOY-03, NFR-SEC-10 · TC-DEPLOY-04–TC-DEPLOY-05 — *in progress — artifacts done: hardened `deploy/container/Dockerfile` (multi-stage, non-root, ca-certs) + `compose.yml` (read-only rootfs, cap_drop ALL, no-new-privileges, private-tunnel publish — NFR-SEC-10) + decision-record skeleton; hadolint CI job. Device-passthrough eval, latency benchmark, and accept/defer decision (BL-092/093/094) are Pi HIL*
- [x] A34. Split-host topology + WireGuard/Tailscale profiles, SSH fallback docs, frontend runtime endpoint config — BL-110–BL-114 · FR-HOST-01–FR-HOST-04, NFR-SEC-13–NFR-SEC-15 · TC-HOST-01–TC-HOST-03, TC-SEC-12–TC-SEC-14 — *done: `deploy/split-host/` — topology + tunnel-interface bind (NFR-SEC-13), WireGuard primary + config templates, Tailscale alternative, SSH fallback-only (NFR-SEC-15), verification checklist. Frontend runtime endpoint config (FR-HOST-04, BL-113) already shipped via `LANDLINE_API_BASE`. On-network verification (TC-HOST/TC-SEC-12–14) is HIL*

## 6. Milestone: Phase 4 — release candidate and operations (forward-looking)

- [x] A35. Define secrets rotation policy and rotation runbook (closes the open BL-012 remainder) — BL-012, BL-105 · NFR-SEC-03 · TC-SEC-03 — *done: security.md §8.2 rotation policy (JWT secret, user hashes, TLS key, tunnel keys — cadences + procedures + immediate-rotation triggers); rotation quick-reference in `deploy/RUNBOOK.md`*
- [x] A36. Production TLS + nginx reverse proxy config — BL-100–BL-101 · NFR-SEC-01 · TC-SEC-01 — *done: `deploy/nginx/nginx.conf` — HTTPS-only (plaintext :80 → 308 redirect, TC-SEC-01), TLS 1.2/1.3 + HSTS, WSS `/ws` upgrade proxying, static frontend, and `X-Forwarded-For` (unblocks the BL-022 rate-limit XFF follow-up). Cert provisioning is deployment-time*
- [ ] A37. Soak test (24 h) and 3-client load test on Pi 4 — BL-102–BL-103 · NFR-REL-03, NFR-PERF-04 · TC-REL-03, TC-PERF-04
- [x] A38. Rollback procedure, ops runbook, final doc alignment, release checklist incl. license compliance — BL-082, BL-104–BL-106, BL-122 · NFR-DEPLOY-05, NFR-LIC-01–NFR-LIC-02 · TC-DEPLOY-06, TC-LIC-01–TC-LIC-02 — *done: `deploy/RUNBOOK.md` (rollback BL-082/NFR-DEPLOY-05, ops runbook BL-105) + `docs/release-checklist.md` (go/no-go gate: traceability, security, deployment, license-compliance BL-122/TC-LIC). BL-104 final trace-alignment stays open until the first release fills the phase execution records from real (HIL) runs*

---

## 7. Definition of done per action

Each action is done per the **Definition of Done in [backlog.md](backlog.md) §1**
(implementation merged, security gate passed where applicable, mapped tests passing, docs
updated). In addition:

- Completing an action must keep the trace gate green (`python3 scripts/trace-gate.py`, or
  `cargo xtask` once promoted per A2).
- Requirements, tests, backlog, and roadmap must be updated **in the same change set** per
  [governance.md](governance.md) §5 change control.

## 8. Change History

| Version | Date | Author | Summary |
|---|---|---|---|
| 0.27 | 2026-07-05 | DC0SK | Could-feature: passband tuning UI (FR-RIG-07) — passband input on the mode control, sent via the existing validated setMode. All Could software features now covered. |
| 0.26 | 2026-07-05 | DC0SK | Could-feature: waterfall palette selection (BL-055, FR-SPEC-04) — hot/grayscale/ice + selector, tested. |
| 0.25 | 2026-07-05 | DC0SK | Opus codec (BL-076 Done): feature-gated libopus OpusCodec (FR-AUD-05); default C-free, CI tests --features opus. 75 Rust tests with feature. |
| 0.24 | 2026-07-05 | DC0SK | Mic TX path: backend Operator-gated TX receive (AudioSink seam); frontend MicCapture + encode + PTT-gated send. Software audio path (RX+TX transport) complete. |
| 0.23 | 2026-07-05 | DC0SK | Frontend RX audio client: TelemetryClient multiplexes spectrum+audio (ADR-02); binary-frame parse + jitter buffer + Web Audio. 35 frontend tests. |
| 0.22 | 2026-07-05 | DC0SK | WS audio RX transport: authenticated binary audio frames over the shared WS (BL-075 Done, BL-071 In Progress). 73 Rust tests. Audio device ends remain HIL. |
| 0.21 | 2026-07-05 | DC0SK | Ops/release docs (A35, A38): secrets rotation policy, ops runbook + rollback, release checklist + license gate. Closes BL-012. All software-buildable work complete; only HIL/browser remains. |
| 0.20 | 2026-07-05 | DC0SK | Closed security remainders + TLS: A36 (nginx TLS/WSS), 0600 config check, panic sanitisation. Retires the NFR-SEC-01/TC-SEC-01 deferral; BL-021 Done. 71 Rust tests. |
| 0.19 | 2026-07-05 | DC0SK | A34 (split-host: WireGuard/Tailscale/SSH profiles + tunnel-bind) done; A33 (container Dockerfile/compose + decision skeleton + hadolint CI) artifacts done. Remaining Phase-3 audio + container eval are Pi HIL. |
| 0.18 | 2026-07-05 | DC0SK | Phase 3 started (A31 in progress): ARC-05 audio software core — jitter buffer (loss concealment) + codec seam + config, C-free. 67 Rust tests. Native/HIL audio parts remain. |
| 0.17 | 2026-07-05 | DC0SK | Phase 2 exit review: reconciled roadmap §6 exit criteria + test-strategy §6b execution record. Phase 2 development-complete; gate pending browser-matrix + Pi HIL. Next milestone Phase 3 (audio). |
| 0.16 | 2026-07-05 | DC0SK | Marked A30 done: audio device selector (MediaDevices) + touch refinements. Phase 2 development-complete; next is the Phase 2 exit review. |
| 0.15 | 2026-07-05 | DC0SK | Marked A29 done: ARC-12 Canvas 2D waterfall (no WebGL) + spectrum-client handshake over the reconnecting socket. 27 frontend tests. Next A30 (browser matrix + audio device selector). |
| 0.14 | 2026-07-05 | DC0SK | Phase 2 started — marked A28 done: ARC-06 FFT pipeline + ARC-01 authenticated WebSocket transport streaming spectrum (also lands deferred WS auth/frame-limit items). 62 Rust tests. Next A29 (Canvas waterfall). |
| 0.13 | 2026-07-05 | DC0SK | Marked A27 done: Phase 1 exit review — reconciled roadmap §5 exit criteria + test-strategy execution record. Phase 1 development-complete; formal gate pending HIL + Phase-4 TLS. |
| 0.12 | 2026-07-05 | DC0SK | Marked A25–A26 done: responsive CSS layout (mobile→3-col, touch, light/dark) and hardened systemd unit + deploy README. Only A27 (Phase 1 exit review) remains. |
| 0.11 | 2026-07-05 | DC0SK | Marked A22–A24 done: mode selector, PTT button (transmit indicator), S-meter display. Full rig-control UI wired; 22 frontend tests. Next action A25 (responsive CSS). |
| 0.10 | 2026-07-05 | DC0SK | Marked A20–A21 done: reconnecting WS client (backoff, NFR-REL-01) + frequency display/tuning UI over REST. 18 frontend tests. Next action A22–A24. |
| 0.9 | 2026-07-04 | DC0SK | Marked A18–A19 done: ARC-10 TypeScript frontend project + authenticated session bootstrap (api/session/backoff + login UI), 11 unit tests, CI frontend job. Next action A20/A21. |
| 0.8 | 2026-07-04 | DC0SK | Marked A15–A17 done: S-meter read endpoint, rig circuit breaker (NFR-REL-02), GPIO control with allowlist + safe states (ARC-08). Phase 1 backend complete; next milestone the frontend (A18+). |
| 0.7 | 2026-07-04 | DC0SK | Marked A11–A14 done: rig control endpoints (frequency, mode, PTT + safety timeout, serialised exclusive access), RBAC-gated + audited; completes TC-AUDIT-01. Next action A15 (S-meter). |
| 0.6 | 2026-07-04 | DC0SK | Marked A10 done: ARC-04 rigctld TCP adapter (typed, validated, injection-proof) + mock-rigctld tests. Next action A11 (frequency handlers). |
| 0.5 | 2026-07-04 | DC0SK | Marked A9 done: ARC-07 tamper-evident audit log (hash chain, auth-failure logging, Admin view). Next action A10 (rigctld adapter). |
| 0.4 | 2026-07-04 | DC0SK | Marked A7/A8 done: ARC-03 security middleware (rate limiting, body-size limit, CORS allowlist). Next action A9 (audit log). |
| 0.3 | 2026-07-04 | DC0SK | Marked A6 done: ARC-02 auth & RBAC (JWT + argon2 + refresh + logout). Recorded ADR-08 (TLS-PSK rejected). Next action A7 (security middleware). |
| 0.2 | 2026-07-04 | DC0SK | Marked A1–A5 done: backend walking skeleton (workspace, server, tracing, config), activated CI/hooks, `cargo xtask`, verified aarch64 cross-build. Next action A6 (auth). |
| 0.1 | 2026-07-04 | DC0SK | Initial action list: Phase 1 kickoff ordering plus forward-looking Phase 2–4 milestones. |
