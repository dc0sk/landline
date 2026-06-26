# landline

A secure, browser-native **web remote for amateur-radio transceivers**. landline runs a
**Rust (Axum/Tokio/Tower) backend on a Raspberry Pi** at the rig site and serves a
**browser-native TypeScript frontend** to any modern desktop or mobile browser — providing
authenticated rig control via hamlib/rigctld, a live spectrum + waterfall display, full-duplex
**Opus** audio, Raspberry Pi GPIO, and split-host operation over a private network — all under an
explicit **security-first, documentation-first** release discipline.

> **Status — documentation / RE baseline complete; implementation not yet started.**
> The requirements-engineering and concept baseline (v0.6 Draft) is in [`docs/`](docs/): vision,
> stakeholder & system requirements, concept/architecture, test strategy, security, governance,
> deployment, backlog and roadmap. **No Rust workspace or frontend code exists yet** — the project
> is at the **Phase 0 → Phase 1** transition. landline is developed **requirements-first and
> test-driven**, with strict traceability from stakeholder needs down to individual test cases,
> enforced by a **build-breaking traceability gate** (rules **R1–R5**, see
> [`docs/README.md`](docs/README.md)).

---

## Documentation

The [`docs/`](docs/) tree is the single source of truth for the requirements-engineering (RE)
and concept phase. **Start with [`docs/README.md`](docs/README.md)** — it defines the RE process,
the document map, the identifier scheme, the requirement attributes, and the R1–R5 traceability
rules.

| Area | Document |
|---|---|
| RE process & ID scheme (entry point) | [`docs/README.md`](docs/README.md) |
| Vision, scope, stakeholders, risks | [`docs/requirements/vision-and-scope.md`](docs/requirements/vision-and-scope.md) |
| Stakeholder requirements (`STK-`) | [`docs/requirements/stakeholder-requirements.md`](docs/requirements/stakeholder-requirements.md) |
| System requirements (`FR-`/`NFR-`) | [`docs/requirements/system-requirements.md`](docs/requirements/system-requirements.md) |
| Concept & architecture (`ARC-`/`ADR-`) | [`docs/concept/architecture.md`](docs/concept/architecture.md) |
| Test strategy & traceability matrix (`TC-`) | [`docs/test/test-strategy.md`](docs/test/test-strategy.md) |
| Security: threat model, controls, gates | [`docs/security.md`](docs/security.md) |
| Governance: phase gates, change control | [`docs/governance.md`](docs/governance.md) |
| Deployment profiles & decision record | [`docs/deployment.md`](docs/deployment.md) |
| Backlog (`EP-`/`BL-`) | [`docs/backlog.md`](docs/backlog.md) |
| Roadmap, milestones, entry/exit gates | [`docs/roadmap.md`](docs/roadmap.md) |

Every Markdown file under `docs/` carries a YAML frontmatter header with the required keys
`title`, `status`, `version`, `updated`, `authors`. The pre-commit hook rejects any `docs/*.md`
missing them.

## Planned architecture

The intended component model is defined in
[`docs/concept/architecture.md`](docs/concept/architecture.md). landline is a two-tier system: a
Rust backend on the Pi at the rig site and a browser-native TypeScript frontend. **None of this is
implemented yet** — the table below is the design-time contract that rule **R5** will check once
code lands.

| Component | Role |
|---|---|
| `ARC-01` Axum HTTP/WS server + Tower middleware | HTTP/WSS endpoints, WebSocket lifecycle, control/audio/spectrum multiplexing and routing |
| `ARC-02` Auth & session | JWT issue/verify, short-lived tokens + refresh, session invalidation, RBAC (Admin/Operator/Observer) |
| `ARC-03` Security middleware | Rate limiting, request/WS-frame size limits, CORS/origin allowlist, error sanitisation |
| `ARC-04` Rig adapter | hamlib/rigctld TCP client, command allowlist + range validation, circuit breaker/timeouts, exclusive access |
| `ARC-05` Audio pipeline | Pi capture/playback, Opus encode/decode, jitter buffering, loss tolerance |
| `ARC-06` Spectrum/FFT pipeline | FFT bin computation, configurable cadence, bounded WS spectrum stream |
| `ARC-07` Audit log subsystem | Tamper-evident, append-only log of state-changing actions and auth failures |
| `ARC-08` GPIO adapter | Allowlisted, role-gated pin read/set with safe default startup states |
| `ARC-09` Config loader | Single secret-free TOML source, 0600 permission checks, no credentials in logs/URLs |
| `ARC-10..12` Frontend app (TypeScript) | Control UI + WS client, audio module (MediaDevices/Opus), Canvas spectrum/waterfall renderer |
| `ARC-13` Deployment artifacts | systemd unit (reference), evaluated container, reverse proxy (TLS), split-host profiles |

## Development / quality gates

landline ships its quality gates **before** the code they will guard, so the requirements
discipline is enforced from day one.

**Runs today (docs-only repo):**

```sh
python3 scripts/trace-gate.py    # requirement -> test traceability gate (R3/R4)
```

The traceability gate parses the requirement tables in
[`docs/requirements/system-requirements.md`](docs/requirements/system-requirements.md) and the
test matrix in [`docs/test/test-strategy.md`](docs/test/test-strategy.md), and **fails** on any
`TC` naming an unknown requirement (**R4**) or any uncovered `M`/`S` requirement (**R3**).

**Git hooks** (enable once per clone):

```sh
git config core.hooksPath .githooks
```

- [`.githooks/pre-commit`](.githooks/pre-commit) — checks that every tracked `docs/**/*.md` has the
  required frontmatter keys, then runs `python3 scripts/trace-gate.py`.
- [`.githooks/pre-push`](.githooks/pre-push) — runs the full `python3 scripts/trace-gate.py`
  (R3/R4) gate.

**CI** is included but **disabled**: [`.github/workflows/ci.yml.disabled`](.github/workflows/ci.yml.disabled).
GitHub Actions only loads `.yml`/`.yaml` files, so it does not run. Enable it with:

```sh
git mv .github/workflows/ci.yml.disabled .github/workflows/ci.yml
```

The enabled workflow runs the same docs frontmatter check and `python3 scripts/trace-gate.py` on
push and pull request.

**Once the Rust workspace lands**, the currently commented-out hook and CI steps activate —
`cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`,
`cargo test --workspace`, and `cargo audit` — and the traceability gate is promoted into
`cargo xtask` per the latent hook steps.

## Deployment

Native **systemd** service is the reference deployment ([`docs/deployment.md`](docs/deployment.md));
a container profile is evaluated (not the default); split-host operation reaches the backend over a
private **WireGuard** or **Tailscale** network (SSH tunnel as fallback only), with the backend bound
to the private interface — never public `0.0.0.0`.

## License

landline is licensed under **AGPL-3.0-only**. See [LICENSE](LICENSE).
