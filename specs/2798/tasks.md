# Tasks: Issue #2798 - PRD Phase 1E responsive sidebar and theme shell controls

## Ordered Tasks
1. [x] T1 (RED): add failing shell contract tests for responsive sidebar + theme markers.
2. [x] T2 (GREEN): expand shell context with theme/sidebar state and add responsive/theme control markup.
3. [x] T3 (GREEN): add gateway integration assertion for phase-1E contract markers on `/ops`.
4. [x] T4 (REGRESSION): run phase-1B/1C/1D regression tests.
5. [x] T5 (VERIFY): run fmt/clippy/scoped tests and set spec status to `Implemented`.
6. [x] T6 (REGRESSION): preserve active session context in sidebar/theme shell controls.

## Tier Mapping
- Unit: shell context/theme/sidebar marker tests in `tau-dashboard-ui`.
- Property: N/A (no randomized invariant domain).
- Contract/DbC: N/A.
- Snapshot: N/A.
- Functional: responsive/theme shell marker assertions.
- Conformance: C-01..C-06 covered by UI and gateway regression tests.
- Integration: gateway `/ops` shell output marker coverage.
- Fuzz: N/A.
- Mutation: N/A (SSR marker contract slice).
- Regression: phase-1B/1C/1D route/auth tests.
- Performance: N/A.

## Verification Evidence

- RED: Live Browser on
  `/ops/login?theme=dark&sidebar=expanded&session=default` showed sidebar and
  theme control hrefs without `session=default`.
- RED: `RUST_MIN_STACK=16777216 cargo test -p tau-dashboard-ui regression_spec_2798_shell_controls_preserve_active_session_context -- --nocapture`
  failed because the visible sidebar control did not preserve session context.
- GREEN: `RUST_MIN_STACK=16777216 cargo test -p tau-dashboard-ui regression_spec_2798_shell_controls_preserve_active_session_context -- --nocapture`
  passed.
- REGRESSION: `RUST_MIN_STACK=16777216 cargo test -p tau-dashboard-ui 2798 -- --nocapture`
  passed (4 tests).
- REGRESSION: `RUST_MIN_STACK=16777216 cargo test -p tau-dashboard-ui 2786_c03 -- --nocapture`
  passed (4 tests).
- GREEN: `RUST_MIN_STACK=16777216 cargo test -p tau-gateway 2798 -- --nocapture`
  passed (2 tests).
- REGRESSION: `RUST_MIN_STACK=16777216 cargo test -p tau-dashboard-ui -- --nocapture`
  passed (197 tests, 0 doc tests).
- STATIC: `cargo fmt --check --package tau-dashboard-ui --package tau-gateway`
  and `git diff --check` passed.
- STATIC: `RUST_MIN_STACK=16777216 cargo clippy -p tau-dashboard-ui -p tau-gateway -- -D warnings`
  passed.
- BUILD: `RUST_MIN_STACK=16777216 cargo build -p tau-coding-agent` passed.
- LIVE: Rebuilt `tau-coding-agent` running on `127.0.0.1:8795` reported
  `auth.mode=localhost-dev`; Browser verified sidebar, dark theme, and light
  theme control hrefs preserve `session=default`.
- LIVE: Browser clicked the visible Light theme control from
  `/ops/login?theme=dark&sidebar=expanded&session=default`; the page reached
  `/ops/login?theme=light&sidebar=expanded&session=default` and reported
  `data-theme=light`.
