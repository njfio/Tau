# Tasks: Issue #2826 - Command-center control confirmation SSR markers

## Ordered Tasks
1. [x] T1 (RED): add failing UI + gateway tests for control confirmation marker contracts.
2. [x] T2 (GREEN): add deterministic confirmation marker attributes to pause/resume/refresh buttons.
3. [x] T3 (REGRESSION): run phase-1A..1K command-center regression suites.
4. [x] T4 (VERIFY): run fmt/clippy/tests/mutation/guardrails and set spec status to `Implemented`.
5. [x] T5 (REGRESSION): make unavailable command-center control actions
   native-disabled instead of clickable submit buttons.

## Tier Mapping
- Unit: N/A.
- Property: N/A.
- Contract/DbC: N/A.
- Snapshot: N/A.
- Functional: control confirmation marker assertions and disabled semantics.
- Conformance: C-01..C-05.
- Integration: gateway `/ops` render marker assertions include disabled
  semantics for unavailable actions.
- Fuzz: N/A.
- Mutation: `cargo mutants --in-diff <diff-file> -p tau-dashboard-ui -p tau-gateway`.
- Regression: phase-1A..1K contract suites.
- Performance: N/A.

## Verification Evidence
- RED:
  - `cargo test -p tau-dashboard-ui functional_spec_2826 -- --test-threads=1` (failed on missing confirmation markers)
  - `cargo test -p tau-gateway functional_spec_2826 -- --test-threads=1` (failed on missing confirmation markers)
- GREEN + regression:
  - `cargo test -p tau-dashboard-ui functional_spec_2826 -- --test-threads=1`
  - `cargo test -p tau-gateway functional_spec_2826 -- --test-threads=1`
  - `cargo test -p tau-dashboard-ui functional_spec_2786 -- --test-threads=1`
  - `cargo test -p tau-dashboard-ui functional_spec_2794 -- --test-threads=1`
  - `cargo test -p tau-dashboard-ui functional_spec_2798 -- --test-threads=1`
  - `cargo test -p tau-dashboard-ui functional_spec_2806 -- --test-threads=1`
  - `cargo test -p tau-dashboard-ui functional_spec_2810 -- --test-threads=1`
  - `cargo test -p tau-dashboard-ui functional_spec_2814 -- --test-threads=1`
  - `cargo test -p tau-dashboard-ui functional_spec_2818 -- --test-threads=1`
  - `cargo test -p tau-dashboard-ui functional_spec_2822 -- --test-threads=1`
  - `cargo test -p tau-dashboard-ui functional_spec_2826 -- --test-threads=1`
  - `cargo test -p tau-gateway functional_spec_2786 -- --test-threads=1`
  - `cargo test -p tau-gateway functional_spec_2794 -- --test-threads=1`
  - `cargo test -p tau-gateway functional_spec_2798 -- --test-threads=1`
  - `cargo test -p tau-gateway functional_spec_2802 -- --test-threads=1`
  - `cargo test -p tau-gateway functional_spec_2806 -- --test-threads=1`
  - `cargo test -p tau-gateway functional_spec_2810 -- --test-threads=1`
  - `cargo test -p tau-gateway functional_spec_2814 -- --test-threads=1`
  - `cargo test -p tau-gateway functional_spec_2818 -- --test-threads=1`
  - `cargo test -p tau-gateway functional_spec_2822 -- --test-threads=1`
  - `cargo test -p tau-gateway functional_spec_2826 -- --test-threads=1`
- Verify:
  - `cargo fmt --check`
  - `cargo clippy -p tau-dashboard-ui -p tau-gateway -- -D warnings`
  - `cargo test -p tau-dashboard-ui`
  - `cargo test -p tau-gateway`
  - `python3 .github/scripts/oversized_file_guard.py`
  - `cargo mutants --in-diff /tmp/mutants_2826.diff -p tau-dashboard-ui -p tau-gateway` (`2/2` caught)
  - `cargo test` (workspace run executed; failures observed in unrelated existing `tau-coding-agent` tests, none in touched crates)

## Follow-up Verification Evidence
- RED: `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-dashboard-ui regression_spec_2826_disabled_control_actions_are_native_disabled -- --nocapture`
  failed because the command-center buttons had `data-action-enabled` markers
  but no `aria-disabled` or native `disabled` semantics.
- GREEN: `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-dashboard-ui regression_spec_2826_disabled_control_actions_are_native_disabled -- --nocapture`
  passed after unavailable controls were rendered as native-disabled buttons.
- SCOPED: `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-dashboard-ui 2826 -- --nocapture`
  passed both command-center control tests.
- INTEGRATION: `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-gateway functional_spec_2826_c03_ops_shell_control_markers_include_confirmation_payload -- --nocapture`
  passed with `/ops` asserting disabled semantics.
- COMMAND CENTER: `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-gateway ops_command_center -- --nocapture`
  passed 18 tests.
- FULL UI: `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-dashboard-ui -- --nocapture`
  passed 205 tests plus doc tests.
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
  `http://127.0.0.1:8795/ops?theme=dark&sidebar=expanded&session=default`;
  the interactive snapshot showed `Resume` as disabled while `Pause` and
  `Refresh` remained enabled, and live HTML exposed
  `aria-disabled="true" disabled` only on the unavailable resume action.
