# Tasks: Issue #2826 - Command-center control confirmation SSR markers

## Ordered Tasks
1. [x] T1 (RED): add failing UI + gateway tests for control confirmation marker contracts.
2. [x] T2 (GREEN): add deterministic confirmation marker attributes to pause/resume/refresh buttons.
3. [x] T3 (REGRESSION): run phase-1A..1K command-center regression suites.
4. [x] T4 (VERIFY): run fmt/clippy/tests/mutation/guardrails and set spec status to `Implemented`.
5. [x] T5 (REGRESSION): make unavailable command-center control actions
   native-disabled instead of clickable submit buttons.
6. [x] T6 (REGRESSION): preserve active shell `session` and timeline `range`
   in command-center control action form submissions.
7. [x] T7 (REGRESSION): ship a browser-native confirmation submit guard for
   confirm-required command-center controls.

## Tier Mapping
- Unit: N/A.
- Property: N/A.
- Contract/DbC: N/A.
- Snapshot: N/A.
- Functional: control confirmation marker assertions, disabled semantics, and
  submit guard marker assertions.
- Conformance: C-01..C-07.
- Integration: gateway `/ops` render marker assertions include disabled
  semantics for unavailable actions and the confirmation guard script.
- Fuzz: N/A.
- Mutation: `cargo mutants --in-diff <diff-file> -p tau-dashboard-ui -p tau-gateway`.
- Regression: phase-1A..1K contract suites.
- Regression: disabled-action and context-preserving form-submit suites.
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

## Context Preservation Follow-up Evidence
- RED: `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-dashboard-ui regression_spec_2826_control_action_forms_preserve_session_and_range_context -- --nocapture`
  failed before implementation because control action forms did not include
  hidden `session` or `range` context fields.
- GREEN: `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-dashboard-ui regression_spec_2826_control_action_forms_preserve_session_and_range_context -- --nocapture`
  passed after pause/resume/refresh forms carried the active session key and
  timeline range.
- SCOPED: `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-dashboard-ui 2826 -- --nocapture`
  passed 3 tests, including the context-preservation regression.
- FULL UI: `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-dashboard-ui -- --nocapture`
  passed 207 tests plus doc tests.
- HYGIENE: `cargo fmt --check --package tau-dashboard-ui --package tau-gateway`,
  `git diff --check`, and
  `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo clippy -p tau-dashboard-ui -p tau-gateway -- -D warnings`
  passed.
- BUILD: `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo build -p tau-coding-agent`
  passed.
- LIVE: restarted `tau-8795` from the rebuilt binary and verified
  `/gateway/status` reported `service_status: running`, auth mode
  `localhost-dev`, and model `gpt-5.3-codex`.
- BROWSER: `agent-browser` followed
  `/ops/login?theme=dark&sidebar=expanded&session=default` -> Continue -> 6h
  -> Refresh and landed on
  `/ops?theme=dark&sidebar=expanded&session=default&range=6h&control_action_status=applied&control_action=refresh&control_action_reason=control_action_applied`.
  Live HTML exposed hidden refresh fields with `session=default` and
  `range=6h`.

## Confirmation Guard Follow-up Evidence
- RED: `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-dashboard-ui regression_spec_2826_control_actions_ship_confirmation_submit_guard -- --nocapture`
  failed because the command-center controls advertised
  `data-confirm-required="true"` but the SSR shell did not ship
  `tau-ops-control-confirmation-guard`.
- GREEN: `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-dashboard-ui regression_spec_2826_control_actions_ship_confirmation_submit_guard -- --nocapture`
  passed after the shell added a browser-native submit guard scoped to
  `tau-ops-control-actions`.
- INTEGRATION: `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-gateway functional_spec_2826_c03_ops_shell_control_markers_include_confirmation_payload -- --nocapture`
  passed with the live `/ops` response asserting the confirmation guard script.
- SCOPED: `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-dashboard-ui 2826 -- --nocapture`
  passed 4 tests.
- COMMAND CENTER: `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-gateway ops_command_center -- --nocapture`
  passed 19 tests.
- FULL UI: `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo test -p tau-dashboard-ui -- --nocapture`
  passed 209 tests plus doc tests.
- HYGIENE: `cargo fmt --check --package tau-dashboard-ui --package tau-gateway`,
  `git diff --check`, and
  `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo clippy -p tau-dashboard-ui -p tau-gateway -- -D warnings`
  passed.
- BUILD: `RUST_MIN_STACK=16777216 CARGO_INCREMENTAL=0 cargo build -p tau-coding-agent`
  passed.
- LIVE: restarted `tau-8795` from the rebuilt binary with
  `RUST_MIN_STACK=16777216`; `/gateway/status` reported
  `service_status=running`, `auth_mode=localhost-dev`, model
  `gpt-5.3-codex`, and state dir `.tau/gateway-live-demo`.
- HTTP: live `/ops?theme=dark&sidebar=expanded&session=default` exposed
  `id="tau-ops-control-confirmation-guard"`,
  `data-confirm-submit-guard="browser-confirm"`,
  `data-confirm-action-scope="tau-ops-control-actions"`, `window.confirm`,
  `event.preventDefault()`, and `data-confirm-result`.
- BROWSER: Browser plugin loaded the rebuilt `/ops` shell and verified
  guard mode `browser-confirm`, guard scope `tau-ops-control-actions`,
  `Pause` and `Refresh` enabled, and `Resume` disabled.
