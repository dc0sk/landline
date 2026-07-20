---
title: "Loose-Ends Audit — 2026-07"
status: Draft
version: "1.0"
updated: 2026-07-21
authors:
  - Simon Keimer (DC0SK)
---

> **Method.** Multi-agent audit across ten dimensions (stubs/dead code, panics in library paths,
> seam gaps, test gaps, concurrency, security, domain correctness, the newest subsystems, frontend,
> docs drift). 64 candidate findings; each was then adversarially verified against the source with
> a refute-by-default instruction. 61 survived (59 confirmed, 2 uncertain), 3 were refuted, and
> deduplication collapsed them to 45 distinct findings.
>
> **Status.** 17 findings are fixed and merged (PRs #37, #39–#46). The remainder are tracked in the
> repository issue that links this document. Two corrections found while fixing are recorded
> against their findings below rather than silently dropped: the Opus `RangeError` blast radius was
> overstated (it is noisy, not fatal), and the audit missed the floating `channel = "stable"`
> toolchain pin entirely — fixed in PR #38.
>
> **Evidence tiers vary and are stated per fix in the PRs.** Several fixes are covered by unit or
> fault-injection tests only; the PTT and audio-device paths still have no hardware-in-the-loop
> evidence.

# Landline — What Isn't Nailed Down
### Independent audit synthesis, adversarially verified

---

## (a) Executive Summary

**Counts.** 64 candidate findings were generated across 10 audit dimensions (stubs/dead-code, panics-in-libs, seam-gaps, test-gaps, concurrency, security, audio-protocol/DSP, GPIO/audio-device, frontend, docs-drift). **61 survived adversarial verification** (59 confirmed, 2 uncertain); **3 were refuted and correctly excluded**. Deduplicating overlapping reports of the same root cause collapses those 61 into **45 distinct findings** below.

**Overall verdict.** The codebase is genuinely solid where it matters most for a first release: RBAC is real and enforced on every HTTP handler, the rigctld command allowlist blocks injection, the audit log is hash-chained and independently verified (GPIO was checked against kernel debugfs, not its own API — exactly the right instinct), and the C-free default-build invariant holds. The verification pass refuted or downgraded the majority of high/critical claims once reachability, requirement scope, and actual transport guarantees were checked against source — this is a team that writes plausible-sounding audit findings that don't all survive scrutiny, which is a healthy sign, not a bad one.

That said, **this is not release-ready**, and the gaps cluster in one exact, self-predicted shape: **controls wired on the HTTP path are routinely missing on the WebSocket path**, and **the PTT safety net's "it unkeys after a timeout" claim rests on a test that would still pass if the actual unkey command were deleted** — which was proven by sabotage during this verification, not merely inferred. A second, independent cluster is **audio sample-rate handling**, which no fewer than four separate audit passes converged on: the rate is never checked against hardware, never checked against config legality, and never communicated to the browser, which hardcodes 48 kHz regardless. A third cluster is **documentation staleness**: the traceability status columns (SRS "Proposed", Test Strategy "Not written") have not moved since before any code landed, even though the release-gate script never reads them — so the gate is real, but the human-facing traceability artifact it sits next to is not trustworthy on its own.

None of the findings below rise to a currently-exploitable remote/unauthenticated vulnerability — the service binds a private interface by design, and the worst security issues require an already-authenticated credential or physical/LAN access. The two RF-safety findings (PTT confirm-gap, shutdown-doesn't-unkey) are the ones that matter operationally and should be fixed before this touches a live transmitter unattended.

---

## (b) Ranked Findings

| # | Finding | Severity | Fix sketch |
|---|---|---|---|
| 1 | PTT safety timer clears "keyed" state before/without confirming rigctld actually unkeyed — on **both** manual PTT-off and the automatic timeout — and the sole regression test asserts an internal flag, not the rig | High [confirmed, sabotage-proven] | Only clear `active`/`generation` on a confirmed `Ok` from `set_ptt(false)`; make the mock rigctld record commands and assert `"T 0"` was sent |
| 2 | WebSocket session authenticates once at connect and never re-checks JWT expiry or revocation — logout and token TTL have zero effect on an open socket | High [confirmed] | Re-run `auth.verify()` inside the `session()` loop on a periodic tick; close on `Err` |
| 3 | `GpiodBackend::write`/`read` discard the ioctl `Result` — a failed pin write still returns 204 and is audited as a success | High [confirmed] | Widen `PinBackend::write`/`read` to return `Result<_, GpioError>` and propagate to the HTTP layer and audit record |
| 4 | Graceful shutdown (SIGTERM) never unkeys PTT; the auto-unkey timer is a bare `tokio::spawn` with no `JoinHandle`, cancelled when the runtime drops | High [confirmed] | Await `ptt.deactivate()` (or a `Drop`/shutdown hook) before returning from `main` |
| 5 | `TelemetryClient` captures the access token once at construction and replays it on every reconnect — the first WS drop after ~15 min of uptime (default TTL) loops forever on a dead token with no auto-recovery | High [confirmed] | Pass `token: () => string`, mirroring the pattern already used correctly by `GpioPanel` |
| 6 | Opus-encoded WS audio frames are reinterpreted as raw `Int16Array` PCM by the browser — no codec field on the wire, no negotiation | High [confirmed] | Add a codec field to the `ready`/frame protocol; ship the matching browser Opus decoder (BL-072) before advertising the `--features opus` Pi build as usable |
| 7 | Audio sample-rate handling is broken end-to-end: cpal opens the device's *default* rate (ALSA prefers 44.1 kHz whenever the range spans it) and discards it; the frontend hardcodes 48 kHz; nothing anywhere compares hardware rate, config rate, and playback rate | High [confirmed] | Request the configured rate explicitly via `supported_input_configs()`, fail loudly on mismatch, and carry the true rate on the wire instead of assuming it client-side |
| 8 | A failed token refresh clears the session but never tears down telemetry — mic capture, RX audio, and the WebSocket (including `sendAudio`) keep running after the UI shows a login screen | High [confirmed] | Call `stopTelemetry()` (which already exists and does the right thing) before `session.clear()` in `maybeRefresh`'s catch arm |
| 9 | No audit log entry exists for anything arriving over the WebSocket — TX audio into the rig and denied TX attempts are both invisible to FR-AUDIT-01 | Medium [confirmed] | Extract `Extension<Arc<AuditLog>>` in `ws::handler`; audit TX-session start/stop and denials, not one event per 20 ms frame |
| 10 | Traceability artifacts are self-contradictory: `trace-gate.py` never reads the Status column, all 74 Test-Strategy TC rows still say "Not written" and all 78 SRS requirements still say "Proposed" while §6 of the same document records many as HIL-pass | Medium [confirmed] | Populate Status from the §6 execution records in the same change set; consider having the gate assert consistency |
| 11 | The WS transmit-audio path — role ACL, decode, and sink — has zero test coverage at any tier; deleting the Operator-role guard leaves the entire suite green (sabotage-verified) | Medium [confirmed, sabotage-proven] | One integration test: Observer sends binary frame → sink receives nothing; Operator sends → it does |
| 12 | Rate limiting is a per-HTTP-request Tower layer; it debits one token for the WS upgrade and never runs again — message rate and concurrent-session count are unbounded for the life of a socket | Medium [confirmed] | Add a per-session token bucket inside the `socket.recv()` arm; cap concurrent sessions with a `Semaphore` |
| 13 | `Argon2::default()` (19 MiB, t=2) runs synchronously on Tokio worker threads with no `spawn_blocking` and no concurrency cap; a handful of concurrent logins stalls unrelated request handling | Medium [confirmed, reproduced] | `spawn_blocking` + a bounding `Semaphore`; also add a dummy-hash branch for unknown usernames (see #14) |
| 14 | Login is a user-enumeration timing oracle: an unknown username returns in ~60 ns, a known one after a full Argon2 verify (~20 ms measured) — ~5 orders of magnitude, trivially readable | Medium [confirmed, reproduced] | Precompute a dummy PHC hash at startup and verify-and-discard on the unknown-user branch |
| 15 | RBAC denials outside the PTT endpoint (GPIO read/set/list, frequency, mode) are never audited — only the one PTT denial call site exists in the whole codebase | Medium [confirmed] | Mirror `control.rs`'s `audit.record_denied` into `gpio.rs::set_pin` and the frequency/mode handlers |
| 16 | Audit log holds a `std::Mutex` across a blocking file write/flush and discards the `io::Error` with `let _ =` — a full disk silently drops audit records while the API returns 204 and the in-memory chain still "verifies" | Medium [confirmed] | Log the `io::Error` at `error!` and expose a dropped-record counter; `spawn_blocking` is optional at this event rate |
| 17 | Circuit breaker is checked *before* queuing on the rig connection mutex, not after — callers already queued behind a dead rigctld pay the full timeout each, serialising the PTT auto-unkey behind the backlog | Medium [confirmed] | Re-check `allow()` after acquiring the connection lock, or `try_lock` + reject |
| 18 | cpal capture ring buffers hit their ~1 s cap within one second of startup and are never drained below it — every spectrum/audio consumer reads a permanent, silent ~1 s-old window for the process lifetime | Medium [confirmed] | Drain to a low-water mark on first subscribe instead of trimming from a fixed 1 s cap; add a dropped-sample counter |
| 19 | cpal stream errors (device unplugged, fatal ALSA error) only `eprintln!` — bypassing `tracing` — and the owning thread parks forever with no recovery; RX/TX silently degrade to injected zeros indistinguishable from a quiet band | Medium [confirmed] | Route through `tracing::error!`; add a liveness counter surfaced on `/healthz` |
| 20 | GPIO hardware-init failure (`--features gpio-device`) silently falls back to `MemoryBackend` — `GET /api/gpio` and every write return success while no real pin ever moves; NFR-SEC-16 safe-state is never applied to hardware | Medium [confirmed] | Fail startup (or expose a degraded flag) when `gpio.enabled` and hardware init fails under the feature build |
| 21 | Audio capture-init failure substitutes a deterministic synthetic tone (only a startup `warn!`) with no client-visible signal distinguishing it from real RF | Medium [confirmed] | Add a `source: "synthetic"|"device"` field to `Ready`/`Spectrum` messages |
| 22 | A configured `capture_device`/`playback_device` substring that matches nothing silently falls back to the host default device with no log line at all | Medium [confirmed] | Treat an unmatched `want` as an error/loud warning; warn on ambiguous multi-match too |
| 23 | Config can hand libopus an illegal `frame_ms`/`sample_rate_hz` combination; `encode` then returns empty payloads forever (silent) and `OpusCodec::new` failure silently downgrades a configured-Opus deployment to uncompressed PCM | Medium [confirmed, reproduced] | Validate frame/rate legality at config load; log the `encode` `Err` arm instead of swallowing it |
| 24 | `get_mode` reuses the write-side 10-token mode allowlist as the read parser — a rig legitimately reporting `PKTFM`, `RTTYR`, etc. makes `GET /api/rig/mode` 400 | Medium [confirmed] | Give the read path a wider representation (e.g. an `Other(String)` variant) without relaxing the write-side allowlist |
| 25 | A rigctld error reply to a multi-line `get` (`m`) stalls the full command timeout instead of failing fast, and can trip the circuit breaker — the connection mutex is held for the whole stall, delaying PTT unkey | Medium [confirmed, reproduced] | Inspect the first line in `exchange`; return the protocol error immediately on an `RPRT` error reply |
| 26 | User manual v1.0 presents receive/transmit audio as unconditionally working; the frontend has no Opus decoder and no browser-tier audio evidence exists anywhere | Medium [confirmed] | Add a caveat to §7/§9 until BL-072/073 land, or gate the claim on the non-opus build |

*(27 further Low-severity findings — dead config keys, a placeholder route, a NaN-triggered panic requiring a hand-edited config, test-evidence gaps that don't affect shipped behavior, and docs-hygiene items — are detailed by area below rather than repeated in this table.)*

---

## (c) Full Detail by Area

### RF Safety / PTT Control

- **PTT unkey is not confirmed before state clears** [confirmed, sabotage-proven] — `backend/src/rig.rs:421-427` (auto safety-timeout task) and `:435-439` (manual `deactivate()`). Both clear `active`/bump `generation` before or regardless of whether `adapter.set_ptt(false)` succeeds; the result is discarded with `let _ =`. Trigger: rigctld link glitch or breaker-open window at the moment PTT-off is requested, or the safety timer firing during the same window. The only test, `backend/tests/rig.rs:91-106`, asserts `!ptt.is_active()` — a flag the code clears unconditionally. Verification **deleted the real unkey call** (`git checkout`-restorable) and the test still passed, exit 0.
- **Shutdown doesn't unkey** [confirmed] — `backend/src/main.rs:38-45`, `backend/src/rig.rs:418-427`. `systemctl stop` / SIGTERM drops the Tokio runtime, cancelling the parked auto-unkey task mid-sleep if PTT happened to be active. `docs/user-manual.md:96` already tells operators not to rely on the timeout — this closes the one case that's cheap to fix.
- **PTT UI/rig desync** [confirmed] — `frontend/src/main.ts:221-303`. `handlePtt`'s UI state only updates on success, so a lost HTTP response leaves the button lying about rig state (self-recovers on next click); closing the tab or signing out mid-transmission sends no `transmit:false` at all, leaving the rig keyed for up to the 120 s server backstop with zero client-side attempt to unkey. Add a `pagehide`/`sendBeacon` unkey and an explicit unkey call in `handleLogout`.
- **WS TX audio not gated on PTT state** [uncertain] — `backend/src/ws.rs:163`. The binary-frame arm checks only `Role::Operator`, never transmit state; no requirement mandates it and no VOX config exists anywhere in the repo, so the "re-keys a VOX rig after timeout" scenario is speculative — but the asymmetry (HTTP has `PttGuard`, WS doesn't) is real and matches the project's own feared bug class.

### Authentication & Session

- **WS never re-validates JWT** [confirmed] — `backend/src/ws.rs:116-201`. `authenticate()` runs once; `claims.exp` and revocation (`Auth::logout`'s `revoked` set) are never re-checked in the session loop. A logged-out or expired credential keeps streaming spectrum/RX audio and feeding the TX sink indefinitely.
- **TelemetryClient stale-token reconnect loop** [confirmed] — `frontend/src/telemetry-client.ts:34`, `frontend/src/main.ts:61-68`. Token is a `readonly` field copied once; every reconnect (AP roam, backend restart, idle proxy) after the 900 s default TTL replays a dead token forever at 30 s backoff, killing waterfall + audio until a manual page reload. `GpioPanel` already does this correctly (`token: () => session.current?.accessToken ?? null`) two lines away.
- **Failed refresh leaves mic/audio/WS running** [confirmed] — `frontend/src/main.ts:305-315`. Catch arm is `session.clear(); render();` — never `stopTelemetry()`. Mic stream, AudioContext, and the WebSocket (including outbound `sendAudio`) survive past the point the UI shows a login screen; a later successful re-login is a silent no-op because `startTelemetry` early-returns while the orphaned client is non-null.
- **Login timing oracle** [confirmed, reproduced] — `backend/src/auth.rs:214-221`. `self.users.get(name).ok_or(...)?` short-circuits before Argon2 runs. Measured 58 ns (unknown) vs 20.3 ms (known, release build) — trivially distinguishable, audited identically as a login failure either way.
- **Argon2 synchronous on Tokio workers** [confirmed, reproduced] — `backend/src/auth.rs:217`, called directly from the async handler with no `spawn_blocking` and no concurrency cap. Reproduced: an unrelated task's scheduling latency rose ~400x under 4 concurrent logins on a 4-worker runtime.
- **MicCapture.stop() race during in-flight start()** [confirmed] — `frontend/src/audio-player.ts:151-178`, `frontend/src/main.ts:221-259`. A `stop()` landing inside the `getUserMedia` await is a no-op; the mic goes live afterward with no held reference at the call site (state self-heals on the next PTT toggle, but the mic is live and unaccounted for in between).
- **refreshAudioDevices unguarded await chain** [confirmed] — `frontend/src/main.ts:101-119`. Fire-and-forget with no `.catch`; `enumerateDevices()` rejection is an unhandled promise rejection. Also opens the mic briefly for every role including Observer purely to unlock device labels (torn down immediately, never streamed — does not contradict the manual).

### Audit & Traceability Integrity

- **GpiodBackend swallows ioctl failures** [confirmed] — `backend/src/gpio.rs:271-280`. `write`/`read` discard the hardware `Result`; a failed pin write on a real Pi still returns 204 and is audited as `gpio.set` success — the tamper-evident log becomes affirmatively wrong for a keying/interlock pin, not merely incomplete.
- **No audit for anything over WebSocket** [confirmed] — `backend/src/ws.rs:98-168`. Handler doesn't even extract `AuditLog`. TX audio into the rig and denied-Observer TX probes both leave zero trace, while the HTTP twin (`POST /api/rig/ptt`) audits both success and denial.
- **RBAC denials outside PTT unaudited** [confirmed] — `backend/src/gpio.rs:329-334`, `backend/src/control.rs:93,127`. `record_denied` has exactly one call site in the whole codebase (PTT). §2.4 of the test strategy defines a security-test pass as "blocked AND audited" — TC-SEC-15 is cited as passing on tests that never look at the audit log.
- **Audit log blocking write + discarded errors** [confirmed] — `backend/src/audit.rs:172-227`. Held mutex, `let _ = writeln!/flush()`. No fsync, so the "blocking" framing is overstated (page-cache write, not a device round-trip), but a disk-full/IO-error condition is silently swallowed with no log line at all, contradicting the module's own "degrade loudly" posture used at open time.
- **Traceability status hygiene / gate is docs-only** [confirmed] — `scripts/trace-gate.py:59-85` never parses the Status column; all 74 `docs/test/test-strategy.md` TC rows read "Not written" while §6 of the same file records many as pass/HIL-pass, and all 78 `docs/requirements/system-requirements.md` requirements still read "Proposed" (file untouched since commit #2, 34 implementation commits ago). A fabricated `TC-FAKE-01 | ... | Pass` row was verified to pass the gate identically to a real one. No runtime impact; a governance/audit-readiness defect.
- **NFR-SEC-09 sanitised-500 test bypasses production wiring** [confirmed] — `backend/tests/security.rs:122-146` builds its own router with the panic layer attached directly, never calling `app()`. Deleting the real `.layer(security::catch_panic_layer())` from `lib.rs:139` leaves the whole suite green (sabotage-verified).
- **TC-SEC-05 (oversized WS frame) claimed evidenced, has no test anywhere** [confirmed] — `docs/test/test-strategy.md` §6b claims "HIL" evidence that doesn't exist in the §6c execution record; the size cap itself (`ws.rs:104-107`) is correctly wired, this is purely an evidence-tier overstatement.

### WebSocket Seam Gaps

- **WS TX path untested at any tier** [confirmed, sabotage-proven] — `backend/src/ws.rs:160-168`. Deleting the `Role::Operator` guard was run against the full suite: exit 0, all green. No test ever sends a binary frame to the server; the only `Binary` reference in `backend/tests/ws.rs` is on the receive side.
- **Rate limiting/session cap absent post-upgrade** [confirmed] — `backend/src/lib.rs:109-117`. `security::rate_limit` runs once, on the HTTP upgrade; the session loop has no per-message budget and there is no cap on concurrent connections (`grep` for `Semaphore`/`max_connections` returns nothing). Requires a valid JWT; the practical cost is CPU (decode + FFT per session), not memory (no unbounded buffer growth found).

### Audio Pipeline

- **Sample rate never validated or negotiated end-to-end** [confirmed] — `backend/src/audio_device.rs:103-106,201-204`, `backend/src/lib.rs:83-89`, `frontend/src/main.ts:59`. cpal opens whatever the device *defaults* to and discards the value; nothing compares it to `config.audio.sample_rate_hz`; the browser hardcodes `new AudioPlayer(48_000)`. Verified in the cpal 0.15.3 ALSA backend that the default device is explicitly steered to 44.1 kHz whenever the hardware range spans it — meaning the mismatch is plausible on ordinary hardware with the shipped no-device-name default config, not an exotic edge case. Silent result: ~8.8% pitch shift, steady underrun padding, and a mislabelled waterfall frequency axis.
- **Opus bytes reinterpreted as PCM** [confirmed] — `frontend/src/audio-player.ts:17`, `backend/src/lib.rs:85-97`. `--features opus` (the documented Pi build) ships audio the browser has no decoder for; `new Int16Array(buffer.slice(8))` on Opus bytes produces noise, and an odd-length packet throws an uncaught `RangeError` that kills the whole telemetry `onmessage` handler (spectrum included, since they're muxed). A live recurrence of the repo's own recorded lesson #3.
- **libopus can be handed illegal params** [confirmed, reproduced] — `backend/src/audio.rs:231`, `backend/src/lib.rs:83-97`. Reproduced: `frame_ms=30` → every encode returns an empty payload (permanent silence, zero log); `sample_rate_hz=44100` → `OpusCodec::new` fails and silently downgrades to uncompressed PCM behind one startup warning.
- **cpal capture rings permanently ~1 s stale** [confirmed] — `backend/src/audio_device.rs:109-117`. Fill from process start regardless of subscribers; trimmed from the front once the ~1 s cap is hit, so every consumer is pinned exactly one second behind live for the process lifetime.
- **cpal runtime stream failure is unobservable** [confirmed] — `backend/src/audio_device.rs:107,205`. Error callbacks are raw `eprintln!` (bypasses `tracing`); the owning thread parks forever with no rebuild path. A mid-session USB-codec disconnect degrades to permanent silence indistinguishable from a quiet band.
- **Audio capture-init failure → synthetic tone** [confirmed] — `backend/src/lib.rs:152-186`. A deterministic two-tone signal (bit-identical every frame) substitutes for a codec not yet enumerated at boot, with only a startup `warn!` and no client-visible flag.
- **Configured device name mismatch falls back silently** [confirmed] — `backend/src/audio_device.rs:38-55`. An unmatched `capture_device`/`playback_device` substring — typo, or codec not attached at boot — silently resolves to the host default with zero log line (worse than the synthetic-tone case, which does at least warn).
- **JitterBuffer / jitter config keys are dead code** [confirmed] — `backend/src/audio.rs:50`, `backend/src/config.rs:50,52`, `backend/src/ws.rs:164`. The backend `JitterBuffer` type is constructed only in its own `#[cfg(test)]` module; `jitter_target_frames`/`jitter_max_frames` are parsed and read nowhere. Impact is largely cosmetic — WS runs over TCP, so reorder/duplication can't occur on this transport, and the frontend has its own working `JitterBuffer` on the RX/playout side. Real residue: two documented config keys that silently do nothing.
- **Spectrum frames carry unused `sample_rate`/`center_hz`; no frequency axis in the UI** [confirmed] — `frontend/src/main.ts:65`, `backend/src/spectrum.rs:44,61`. The analyzer is single-sided baseband ([0, fs/2)), not IQ centred on `center_hz` — the field name would mislead a future consumer that tries to use it for centring. No requirement currently promises a frequency scale, so this is a UX/protocol-cleanliness gap, not a broken feature.
- **User manual overclaims browser audio** [confirmed] — `docs/user-manual.md:110,117`, no caveat anywhere in §7/§9. Directly follows from the Opus-as-PCM finding above; the backlog (`BL-072/073: In Progress`) is honest, the operator-facing manual is not.

### GPIO

- **GPIO hardware-init failure silently degrades to MemoryBackend** [confirmed] — `backend/src/gpio.rs:129-143`. Any `GpiodBackend::new` failure (wrong chip path — notably Pi 5's header is `/dev/gpiochip4`, not the default `gpiochip0` — permission error, line contention) falls through to an in-memory simulator that answers every read/write as if it worked, with only a startup `warn!`. NFR-SEC-16's safe-startup guarantee is not applied to any real pin in this state.

### Rig Protocol (rigctld)

- **`get_mode` reuses the write-side allowlist as a read parser** [confirmed] — `backend/src/rig.rs:66-101,251`. hamlib's mode vocabulary is much larger than the 10-token write allowlist; a rig legitimately in e.g. `PKTFM` makes a pure read (`GET /api/rig/mode`) fail with 400.
- **Multi-line get stalls on an error reply** [confirmed, reproduced] — `backend/src/rig.rs:250,341-369`. `get_mode` expects 2 lines; a single-line `RPRT -x` error leaves the second `read_line` pending for the full command timeout, and the connection mutex is held the whole time, delaying anything queued behind it including PTT unkey. Reproduced: 3 such errors trip the breaker and a subsequent unrelated `get_frequency` fails fast with `Unavailable`.
- **Rig mode allowlist is a hand-maintained 3-way mirror with no completeness test** [confirmed] — `backend/src/rig.rs:66-99`, `frontend/src/control.ts:26-37`. Currently in sync (verified byte-for-byte); deleting one arm from `Mode::parse` leaves the whole suite green.
- **Circuit breaker checked before, not after, queuing on the connection mutex** [confirmed] — `backend/src/rig.rs:291-314`. A burst of callers against a dead rigctld all pass the closed-breaker check and then serialise, so caller N waits ~N × timeout instead of getting a fast 503.

### Concurrency / Resource Exhaustion

- **Rate-limiter bucket map never evicted** [confirmed] — `backend/src/security.rs:35,62`. Unlike `Auth::is_revoked`'s `retain`, no sweep/cap exists; needs a large number of distinct source IPs to matter, and the documented deployment postures (loopback bind, WireGuard/Tailscale, reverse proxy) all bound the practical source-IP set.
- **ServeDir static-file fallback serves the whole configured directory outside auth and rate limiting, and doesn't filter dotfiles** [confirmed] — `backend/src/lib.rs:121-126`, `backend/config.example.toml:16`. The shipped example points `static_dir` at the frontend *source* tree; `.git` metadata is not reachable in that exact layout (it's a directory above the served root, and `..` is blocked), but `node_modules`, `tsconfig.json`, and `package-lock.json` are.

### Stubs / Dead Code / Minor Hygiene

- **`retention_days` config field is inert** [confirmed] — `backend/src/config.rs:167`. No code reads it; the app never rotates or prunes the audit log (unbounded growth on an SD card), and no logrotate/systemd artifact ships to back the "enforced by deployment" doc claim.
- **`/api/operator-ping` demo placeholder route ships in production** [confirmed] — `backend/src/auth.rs:367`. Stale "seam for real routes" comment; only consumed by two of its own tests, which duplicate coverage already proven against real routes in `control.rs`/`gpio.rs`.
- **NaN `spectrum.update_rate_hz` panics every WS session outside the panic-catch layer** [confirmed, reproduced] — `backend/src/ws.rs:132-133`. `f32::clamp` passes NaN through; `Duration::from_secs_f32(NaN)` panics inside the WS upgrade's detached spawned task, which the outer `CatchPanicLayer` cannot see. Requires an operator to literally type `nan` into a 0600 config file — not attacker-reachable.
- **NoopSink silently discards Operator mic-TX audio by default** [uncertain] — `backend/src/audio.rs:150-154`, `backend/src/lib.rs:185`. Only the default (non-`audio-device`) build; that build also runs a synthetic RF source and a memory GPIO backend, so an operator would notice the whole rig is fake, not just TX audio.
- **README claims CI is disabled and the Rust workspace doesn't exist** [confirmed] — `README.md:74-123`. Both are false: `.github/workflows/ci.yml` is live with four real jobs (clippy `-D warnings`, tests, `cargo audit`, aarch64 cross-build, frontend typecheck/test/build) and has been since the second commit. The gate itself is fine; the README that describes it to new contributors is stale in the safe direction (undersells rather than oversells).
- **`action-list.md` stale by several shipped commits** [confirmed] — `docs/action-list.md:105,130`. Still lists the CPAL audio adapter and gpiod GPIO backend as open HIL work; both are merged and marked Done in `docs/backlog.md`.
- **`GET /api/gpio` shipped with no doc mention and no integration test** [confirmed] — `backend/src/gpio.rs:289`. The role gate is present and matches its tested siblings byte-for-byte; the gap is purely an HTTP-layer test and a one-line docs update.
- **`backlog.md` content changed without a version bump or changelog row** [confirmed] — `docs/backlog.md:4`. A sibling doc (`test-strategy.md`) was bumped correctly in the same commit; pure bookkeeping, no downstream effect since the gate doesn't check this either.

---

## (d) Deferred Tail — Feature Work / Design Decisions, Not Defects

- **Browser Opus decode/encode** (BL-072/073, tracked "In Progress" in the backlog) is genuinely unbuilt, not regressed. It is listed as a *defect* above (#6) only because the operator-facing manual and the `--features opus` build instructions currently claim it works when it doesn't yet — the underlying capability gap itself belongs here.
- **Spectrum frequency-axis / IQ-centring UI.** No requirement currently asks for a frequency scale on the waterfall; `center_hz` is documented as passthrough metadata. Worth a backlog item, not a bug.
- **Backend `JitterBuffer` reorder/loss-concealment logic.** Its premise (packet reorder/loss) doesn't apply over WebSocket-over-TCP today. Either remove the dead struct and its two config keys, or repurpose it for a future datagram/WebTransport audio path — a design decision, not something broken now.
- **Audit `retention_days` / log rotation.** The intended design ("enforced by deployment log rotation") is a reasonable split of responsibility; the gap is that no logrotate/systemd artifact ships to actually realize it. Worth shipping the artifact rather than treating the config key as a defect in the app itself.
- **`refreshAudioDevices` opening the mic for Observer accounts to read device labels.** Torn down immediately, never streamed, and consistent with how the manual describes it — a minor UX/permissions-prompt annoyance, not a privacy defect.