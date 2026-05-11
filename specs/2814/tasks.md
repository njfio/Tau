# Tasks: Issue #2814 - Command-center timeline chart and range SSR markers

## Ordered Tasks
1. [x] T1 (RED): add failing `/ops` integration tests for timeline chart metadata and range selector markers.
2. [x] T2 (GREEN): extend `tau-dashboard-ui` command-center snapshot/markup for timeline chart + range selector markers.
3. [x] T3 (GREEN): extend ops shell controls range query parsing and gateway mapping.
4. [x] T4 (REGRESSION): run phase-1A..1H regression suites.
5. [x] T5 (VERIFY): run fmt/clippy/tests/mutation/guardrails and set spec status to `Implemented`.
6. [x] T6 (REGRESSION): preserve active session context in timeline range
   controls.

## Tier Mapping
- Unit: UI marker rendering + range parser behavior.
- Property: N/A.
- Contract/DbC: N/A.
- Snapshot: N/A.
- Functional: timeline chart/range marker assertions and session-preserving
  range hrefs.
- Conformance: C-01..C-05.
- Integration: gateway `/ops` render with dashboard fixtures and session-scoped
  range query.
- Fuzz: N/A.
- Mutation: `cargo mutants --in-diff <diff-file> -p tau-gateway -p tau-dashboard-ui`.
- Regression: phase-1A..1H contract suites.
- Performance: N/A.

## Verification Evidence
- `cargo fmt --check` ✅
- `cargo clippy -p tau-dashboard-ui -p tau-gateway -- -D warnings` ✅
- `cargo test -p tau-dashboard-ui functional_spec_2814 -- --test-threads=1` ✅
- `cargo test -p tau-gateway functional_spec_2814 -- --test-threads=1` ✅
- `cargo test -p tau-gateway functional_spec_{2786,2794,2798,2802,2806,2810} -- --test-threads=1` ✅
- `cargo test -p tau-dashboard-ui` ✅
- `cargo test -p tau-gateway` ✅
- `python3 .github/scripts/oversized_file_guard.py` ✅
- `cargo mutants --in-diff /tmp/mutants_2814.diff -p tau-gateway -p tau-dashboard-ui` ✅ (`17 tested, 11 caught, 6 unviable, 0 escaped`)

## Follow-up Verification Evidence
- RED: Live Browser clicked `6h` from
  `/ops?theme=dark&sidebar=expanded&session=default` and reached
  `/ops?theme=dark&sidebar=expanded&range=6h`, proving the timeline range
  controls dropped session context.
- RED: `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-dashboard-ui regression_spec_2814_timeline_range_controls_preserve_session_context -- --nocapture`
  failed while range hrefs omitted `session`.
- GREEN: `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-dashboard-ui regression_spec_2814_timeline_range_controls_preserve_session_context -- --nocapture`
  passed after range hrefs reused shell context and appended `range`.
- INTEGRATION: `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-gateway functional_spec_2814_c01_c02_ops_shell_timeline_chart_markers_reflect_snapshot_and_range_query -- --nocapture`
  passed with `/ops?session=ops-live-session&range=6h` preserving session in
  all timeline range hrefs.
- SCOPED: `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-dashboard-ui 2814 -- --nocapture`
  passed both timeline range tests.
- SCOPED: `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-gateway functional_spec_2814 -- --nocapture`
  passed both gateway timeline tests.
- COMMAND CENTER: `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-gateway ops_command_center -- --nocapture`
  passed 18 tests.
- FULL UI: `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-dashboard-ui -- --nocapture`
  passed 206 tests plus doc tests.
- HYGIENE: `cargo fmt --check --package tau-dashboard-ui --package tau-gateway`,
  `git diff --check`, and
  `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo clippy -p tau-dashboard-ui -p tau-gateway -- -D warnings`
  passed.
- BUILD: `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo build -p tau-coding-agent`
  passed.
- LIVE: restarted `tau-8795` from the rebuilt binary and verified
  `http://127.0.0.1:8795/gateway/status` reported `service_status: running`,
  `auth mode: localhost-dev`, and `model: gpt-5.3-codex`.
- BROWSER: `agent-browser` opened
  `http://127.0.0.1:8795/ops/login?theme=dark&sidebar=expanded&session=default`,
  clicked `Continue`, then clicked `6h`; the browser reached
  `http://127.0.0.1:8795/ops?theme=dark&sidebar=expanded&session=default&range=6h`.
- LIVE HTML: `/ops?theme=dark&sidebar=expanded&session=default&range=6h`
  rendered 1h/6h/24h timeline hrefs with `session=default` preserved.
