# Tasks: Issue #3426 - Auth lifecycle verification and hardening

- [x] T1 (RED, Functional/Conformance): add failing tests for missing gateway auth lifecycle conformance cases (password-session bootstrap contract, auth-session mode mismatch, malformed auth-session body, status telemetry counters, bearer scheme casing).
- [x] T2 (GREEN, Implementation): apply minimal gateway auth hardening/fixes required by T1 failures.
- [x] T3 (REGRESSION, Functional/Integration): re-run existing gateway auth/session and PRD-mapped auth selectors to confirm no regressions.
- [x] T4 (VERIFY, Conformance): capture AC -> test evidence and update issue/spec status to implemented state.

## Tier Mapping
- Unit: gateway helper logic covered through focused auth tests.
- Property: N/A (no probabilistic parser/invariant extension in this slice).
- Contract/DbC: N/A (no new DbC annotations).
- Snapshot: N/A.
- Functional: bootstrap/session/status auth workflows.
- Conformance: C-01..C-11 mapped in test names and verify commands.
- Integration: protected endpoint calls with issued/expired tokens.
- Fuzz: N/A (no new untrusted parser surface added).
- Mutation: N/A (non-critical-path docs/test slice; no mutation gate required by issue policy).
- Regression: existing gateway auth/session selectors.
- Performance: N/A (no hot-path performance changes).

## Verification Evidence (2026-02-23)

### RED
- `cargo test -p tau-gateway spec_3426 -- --test-threads=1`
- Result:
  - `regression_spec_3426_c08_gateway_accepts_lowercase_bearer_authorization_scheme` failed (`401` vs expected `200`).
  - initial C10 counter assertion failed due test assumption mismatch (`active_sessions` expected `1`, observed `2`).

### GREEN
- Implemented changes:
  - `crates/tau-gateway/src/gateway_openresponses/auth_runtime.rs`
    - `Authorization` bearer scheme parsing now accepts case-insensitive `Bearer` token scheme while preserving fail-closed token validation.
  - `crates/tau-gateway/src/gateway_openresponses/tests.rs`
    - added C02/C06/C07/C08/C10 auth lifecycle tests.
    - corrected C10 counter assertion to match helper-configured TTL contract.
- `cargo test -p tau-gateway spec_3426 -- --test-threads=1` -> `5 passed; 0 failed`.

### REGRESSION
- `cargo test -p tau-gateway auth_session -- --test-threads=1` -> `4 passed; 0 failed`.
- `cargo test -p tau-gateway spec_2786_c0 -- --test-threads=1` -> `3 passed; 0 failed`.
- `cargo fmt --all -- --check` -> passed.
- `cargo clippy -p tau-gateway --tests -- -D warnings` -> passed.
