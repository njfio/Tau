# Tasks: Issue #2786 - PRD Phase 1B auth bootstrap and protected route shell markers

## Ordered Tasks
1. [x] T1 (RED): add failing tests for auth bootstrap endpoint and auth-aware `/ops`/`/ops/login` shell markers.
2. [x] T2 (GREEN): implement `tau-dashboard-ui` auth/route context render contracts and unit coverage.
3. [x] T3 (GREEN): implement gateway auth bootstrap endpoint and route wiring for `/ops/login` + context-aware `/ops`.
4. [x] T4 (REGRESSION): run existing `/dashboard` and `/gateway/auth/session` regression tests.
5. [x] T5 (VERIFY): run scoped fmt/clippy/tests and set spec status to `Implemented`.
6. [x] T6 (REGRESSION): make localhost-dev `/ops/login` Continue route-backed
   instead of inert, preserving theme/sidebar/session context.
7. [x] T7 (REGRESSION): make localhost-dev `/ops/login` help copy explicitly
   no-auth instead of generic gateway-auth wording.

## Tier Mapping
- Unit: `tau-dashboard-ui` auth marker tests.
- Property: N/A (no parser/invariant expansion in this slice).
- Contract/DbC: N/A (no new DbC macro surfaces).
- Snapshot: N/A.
- Functional: auth bootstrap JSON contract tests.
- Conformance: C-01..C-07 mapped in crate/gateway tests.
- Integration: `/ops` and `/ops/login` endpoint tests.
- Fuzz: N/A (no untrusted parser added).
- Mutation: N/A (scaffolding contract slice; no critical algorithm path).
- Regression: existing `/dashboard` and auth session tests.
- Performance: N/A (no hot-path runtime changes).

## Verification Evidence

- RED: Live Browser on
  `/ops/login?theme=dark&sidebar=expanded&session=default` found one visible
  `Continue` button, clicked it, and remained on `/ops/login`, proving the
  control was inert in localhost-dev/no-auth mode.
- GREEN: `RUST_MIN_STACK=16777216 cargo test -p tau-dashboard-ui 2786_c03 -- --nocapture`
  passed with no-auth Continue markers and existing auth shell coverage (4
  tests).
- GREEN: `RUST_MIN_STACK=16777216 cargo test -p tau-gateway ops_login -- --nocapture`
  passed with the localhost-dev `/ops/login` Continue href contract (2 tests).
- REGRESSION: `RUST_MIN_STACK=16777216 cargo test -p tau-dashboard-ui -- --nocapture`
  passed (196 tests, 0 doc tests).
- REGRESSION: `RUST_MIN_STACK=16777216 cargo test -p tau-gateway ops_auth_navigation -- --test-threads=1 --nocapture`
  passed (9 tests).
- STATIC: `cargo fmt --check --package tau-dashboard-ui --package tau-gateway`
  and `git diff --check` passed.
- STATIC: `RUST_MIN_STACK=16777216 cargo clippy -p tau-dashboard-ui -p tau-gateway -- -D warnings`
  passed.
- BUILD: `RUST_MIN_STACK=16777216 cargo build -p tau-coding-agent` passed.
- LIVE: Rebuilt `tau-coding-agent` running on `127.0.0.1:8795` reported
  `auth.mode=localhost-dev`; Browser verified the visible Continue link at
  `/ops/login?theme=dark&sidebar=expanded&session=default` navigated to
  `/ops?theme=dark&sidebar=expanded&session=default` and exposed the command
  center with `aria-hidden=false`.
- RED: Live Browser on
  `/ops/login?theme=dark&sidebar=expanded&session=default` reported
  `auth.mode=localhost-dev`, `data-login-required=false`, and disabled auth
  input markers, but the help copy still said `Use configured gateway auth
  mode to continue to protected operations views.`
- RED: `RUST_MIN_STACK=16777216 cargo test -p tau-dashboard-ui regression_spec_2786_none_login_copy_does_not_imply_auth_is_required -- --nocapture`
  failed before the auth-mode-specific help text was added.
- GREEN: `RUST_MIN_STACK=16777216 cargo test -p tau-dashboard-ui regression_spec_2786_none_login_copy_does_not_imply_auth_is_required -- --nocapture`
  passed.
- REGRESSION: `RUST_MIN_STACK=16777216 cargo test -p tau-dashboard-ui 2786 -- --nocapture`
  passed (5 tests).
- REGRESSION: `RUST_MIN_STACK=16777216 cargo test -p tau-gateway ops_login -- --nocapture`
  passed (2 tests).
- REGRESSION: `RUST_MIN_STACK=16777216 cargo test -p tau-dashboard-ui -- --nocapture`
  passed (198 tests, 0 doc tests).
- STATIC: `cargo fmt --check --package tau-dashboard-ui --package tau-gateway`
  and `git diff --check` passed.
- STATIC: `RUST_MIN_STACK=16777216 cargo clippy -p tau-dashboard-ui -p tau-gateway -- -D warnings`
  passed.
- BUILD: `RUST_MIN_STACK=16777216 cargo build -p tau-coding-agent` passed.
- LIVE: Rebuilt `tau-coding-agent` running on `127.0.0.1:8795` reported
  `auth.mode=localhost-dev`; Browser verified
  `id=tau-ops-login-help`, `data-auth-copy-mode=none`, copy text
  `Localhost-dev mode is active. No credential is required; continue directly
  to protected operations views.`, and no generic gateway-auth copy.
