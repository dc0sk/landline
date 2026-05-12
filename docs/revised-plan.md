---
title: Secure-First Revised Plan
project: landline
doc_type: implementation-plan
status: draft
version: 0.3.0
owner: ""
last_updated: 2026-05-12
security_first: true
container_evaluation: true
documentation_first: true
---

## Plan: Secure Hamradio Web Remote

Build a Rust-based web remote system for ham transceivers where security is a release gate from day one, not a final hardening step. Use a browser-native frontend (no egui for MVP), an Axum/Tokio backend on Raspberry Pi, and evaluate container deployment in parallel with native systemd deployment before committing to production packaging.

**Documentation baseline (mandatory before implementation)**
- Maintain four core project artifacts from the start:
  - Requirements specification document (functional, non-functional, security, deployment, compatibility).
  - Test specification document (test strategy, levels, traceability matrix, acceptance criteria).
  - Product backlog (epics, features, user stories, priorities, dependencies, definition of done).
  - Roadmap and release/phase plan (milestones, scope per phase, entry/exit gates, release criteria).
- Keep all artifacts versioned in docs and updated at every phase gate.

**Steps**
1. Establish project documentation framework and governance first.  
Dependencies: blocks all implementation and design decisions.  
Actions:
- Create docs/requirements-spec.md as the source of truth for product and engineering requirements.
- Create docs/test-spec.md including requirement-to-test traceability IDs.
- Create docs/backlog.md with prioritization model (Must/Should/Could/Won't or equivalent).
- Create docs/roadmap.md with release/phase plan, milestones, and scope boundaries.
- Define change control: any scope, security, or architecture change must update requirements, tests, backlog, and roadmap together.

2. Define non-negotiable security baseline and threat model before feature work.  
Depends on 1.  
Actions:
- Document trust boundaries: browser client, backend service, rig interface, audio path, reverse proxy, LAN/WAN.
- Classify deployment modes: LAN-only, VPN-only remote, internet-exposed.
- Set security gates required for MVP release: HTTPS/WSS only, authenticated control plane, role/permission checks, audit logging, input validation for rig commands, and safe defaults.
- Define secrets handling and rotation policy (token/key storage, file permissions, rotation cadence).

3. Finalize architecture and transport with security-first constraints.  
Depends on 1 and 2.  
Actions:
- Choose WebSocket binary transport for control/audio/spectrum with explicit message schemas and bounds checking.
- Use strict server-side validation for frequency/mode/PTT requests.
- Separate control and telemetry channels logically (can share connection but enforce message type ACLs).
- Define reconnection/session semantics that do not bypass auth.

4. Build backend foundation (Rust on Raspberry Pi) with security middleware first.  
Depends on 3.  
Actions:
- Backend stack: Tokio + Axum + Tower middleware + Tracing.
- Implement auth middleware before control handlers (token/JWT, expiry, optional refresh).
- Add rate limiting, request size limits, websocket frame limits, and CORS/origin policy.
- Add tamper-evident audit log events for rig-changing actions.
- Implement rig access adapter via hamlib/rigctld with command sanitization and timeout/circuit-breaker behavior.

5. Build frontend with browser-wide compatibility and explicit security UX.  
Parallel with late part of 4 after API/auth contracts are stable.  
Actions:
- Use TypeScript web frontend (responsive desktop/mobile layouts).
- Ensure compatibility tests for Firefox, Chromium, iOS Safari, Android browsers.
- Implement authenticated session bootstrap and clear auth/error states.
- Add local audio device selection UI using Web Audio/media devices APIs.
- Implement spectrum/waterfall visualization with bounded frame/update rates.

6. Implement audio pipeline with secure transport and operational safeguards.  
Depends on 4 and 5.  
Actions:
- Capture/playback via browser media APIs; backend audio capture/playback on Pi side.
- Use encrypted transport (WSS), per-session auth checks for audio channels.
- Apply bitrate/sample-rate profiles for constrained mobile clients.
- Add drop/retry logic and watchdogs to avoid backend lockups.

7. Containerization evaluation and deployment decision.  
Parallel with 4-6 once baseline service is runnable.  
Actions:
- Build and benchmark two deployment modes:
  - Native: systemd service on Raspberry Pi OS.
  - Containerized: rootless container where feasible, host networking only when required, persistent volumes for config/logs/certs.
- Evaluate container viability against hardware/audio needs:
  - Access to ALSA/Pulse/PipeWire devices.
  - Access to serial/USB/GPIO (if used by radio interface).
  - Realtime/latency impact for audio.
  - Update/rollback workflow.
- Security comparison checklist:
  - Image provenance and patch cadence.
  - Dropped Linux capabilities, readonly rootfs, non-root user.
  - Secret injection method.
  - Network exposure model and reverse proxy integration.
- Decision output:
  - Keep native as reference deployment if device access/latency is superior.
  - Offer container deployment profile if performance/security acceptance criteria pass.

8. Verification and release gating.  
Depends on 1-7.  
Actions:
- Security tests first: unauthorized command attempts, replay attempts, malformed websocket frames, rate-limit abuse, token expiry behavior.
- Cross-browser/device validation matrix: Firefox, Chromium, iOS Safari, Android browsers.
- Raspberry Pi 4/5 load and soak tests (control latency, audio stability, spectrum cadence, thermal budget).
- Deployment verification for both native and container mode.
- Define go/no-go criteria with explicit thresholds and required test pass set.

9. Documentation-driven release management.  
Depends on 1-8 and is enforced at every phase boundary.  
Actions:
- Update docs/requirements-spec.md with implemented scope, deferred scope, and requirement status.
- Update docs/test-spec.md with executed test evidence and pass/fail by requirement ID.
- Re-prioritize docs/backlog.md using test outcomes, security findings, and performance data.
- Update docs/roadmap.md with next phase entry criteria and release date confidence.

**Roadmap and release/phase plan**
1. Phase 0 - Foundation and documentation
- Deliverables: requirements spec v1, test spec v1, backlog v1, roadmap v1, threat model v1.
- Exit criteria: documentation baseline approved; security gates defined.
2. Phase 1 - Secure control MVP
- Deliverables: authenticated rig control UI + backend control APIs + audit logs.
- Exit criteria: control latency target met; unauthorized control attempts blocked.
3. Phase 2 - Spectrum/waterfall and mobile compatibility
- Deliverables: responsive UI, spectrum pipeline, mobile browser support validation.
- Exit criteria: browser matrix pass for Firefox/Chromium/iOS/Android on defined test set.
4. Phase 3 - Audio and deployment hardening
- Deliverables: bidirectional audio path, TLS production setup, container evaluation report.
- Exit criteria: stable long-run audio sessions; native vs container decision recorded.
5. Phase 4 - Release candidate and operations
- Deliverables: release checklist, deployment runbooks, rollback plan, final docs alignment.
- Exit criteria: all critical requirements traced to passing tests; go/no-go approval complete.

**Backlog management model**
- Structure: Epic -> Feature -> User Story -> Task.
- Priority classes: Must, Should, Could, Won't (per release).
- Required fields per item: ID, owner, estimate, dependency, risk, acceptance criteria, linked requirement IDs, linked test IDs.
- Definition of done: implementation complete, security checks passed, tests added/executed, documentation updated.

**Relevant files**
- docs/requirements-spec.md - formal requirements specification with IDs and status.
- docs/test-spec.md - test strategy, test catalog, and requirement traceability.
- docs/backlog.md - prioritized product/engineering backlog with dependencies.
- docs/roadmap.md - release/phase planning, milestones, and gate criteria.
- backend/src/main.rs - API bootstrap, middleware, websocket lifecycle.
- backend/src/auth.rs - authn/authz policies, token validation, session guards.
- backend/src/security.rs - rate limits, input/frame validation, origin policy.
- backend/src/rig_adapter.rs - hamlib/rigctld integration and command validation.
- backend/src/audio.rs - audio session transport and codec pipeline.
- frontend/src/app.ts - control UI, session/auth handling, websocket client.
- frontend/src/audio.ts - browser media device selection and audio I/O.
- frontend/src/spectrum.ts - waterfall rendering and update throttling.
- deploy/systemd/landline.service - native service deployment.
- deploy/container/Dockerfile - container image build.
- deploy/container/compose.yml - local orchestration with volumes/network.
- deploy/nginx/nginx.conf - TLS termination, reverse proxy, websocket headers.
- docs/security.md - threat model, controls, incident/rotation procedures.
- docs/deployment.md - native vs container decision record and runbooks.

**Verification**
1. Verify each requirement ID has at least one mapped test in docs/test-spec.md.
2. Run backend tests and static checks (unit + integration + lint) with security tests as mandatory blockers.
3. Execute browser compatibility test suite across desktop/mobile targets and verify audio device selection flows.
4. Perform websocket fuzz/malformed-frame tests and confirm no panic, no unauthorized state mutation.
5. Validate TLS/auth/rate-limit behavior behind reverse proxy and in direct LAN mode.
6. Benchmark native vs container on Pi 4 and Pi 5: CPU, memory, end-to-end audio latency, reconnect behavior, and long-run stability.
7. Approve deployment mode only if it passes defined latency/security thresholds.
8. Block release if backlog items marked Must do not meet acceptance criteria or documentation traceability is incomplete.

**Decisions**
- Security-first constraint is mandatory: no feature considered complete without passing defined security gate.
- egui is excluded from MVP because browser-native TS frontend provides better mobile/browser fit and lower operational risk.
- Container deployment is in-scope as an evaluated delivery option, not automatically the default runtime.
- Raspberry Pi 4/5 + Raspberry Pi OS remains the reference target; architecture should keep portability for future targets.
- Documentation-first constraint is mandatory: requirements, tests, backlog, and roadmap are release artifacts, not optional notes.

**Further considerations**
1. Auth model choice for MVP: Option A static bearer tokens (simpler), Option B short-lived JWT with refresh (stronger session control). Recommendation: Option B if internet exposure is expected in first release.
2. Remote access stance: Option A LAN/VPN-only for MVP, Option B direct internet exposure behind hardened reverse proxy. Recommendation: Option A for MVP risk reduction.
3. Container runtime target: Option A Docker/Compose (common), Option B Podman rootless (security-oriented). Recommendation: test both, prefer rootless if hardware/audio access constraints are acceptable.
