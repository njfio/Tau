# Tasks: Issue #2838 - Sessions explorer deterministic row contracts

## Ordered Tasks
1. [x] T1 (RED): add conformance tests for `/ops/sessions` panel/list/row/empty-state SSR markers in UI and gateway.
2. [x] T2 (GREEN): implement sessions explorer snapshot/render mapping in UI + gateway.
3. [x] T3 (REGRESSION): run phase 1N chat selector suites and ops route marker regressions.
4. [x] T4 (VERIFY): run fmt/clippy/tests/guardrails/mutation and set spec status `Implemented`.
5. [x] T5 (RED/GREEN/REGRESSION): make sessions explorer rows open session-detail routes instead of switching chat sessions.

## Tier Mapping
- Unit: selector/sessions helper behavior where isolated.
- Property: N/A.
- Contract/DbC: N/A.
- Snapshot: N/A.
- Functional: `/ops/sessions` marker contracts.
- Conformance: C-01..C-04.
- Integration: discovered session rows + preserved row href controls.
- Fuzz: N/A.
- Mutation: `cargo mutants --in-diff <diff-file> -p tau-dashboard-ui -p tau-gateway`.
- Regression: phase 1N and route-shell suites.
- Performance: N/A.

## Verification Evidence
- Targeted:
  - `cargo test -p tau-dashboard-ui functional_spec_2838 -- --test-threads=1`
  - `cargo test -p tau-gateway functional_spec_2838 -- --test-threads=1`
  - `cargo test -p tau-gateway integration_spec_2838 -- --test-threads=1`
  - RED: `env RUST_MIN_STACK=16777216 cargo test -p tau-dashboard-ui functional_spec_2838_c01_c02_c03_sessions_route_renders_sessions_panel_list_rows_and_links -- --test-threads=1` failed while rows still rendered `/ops/chat?...session=...` links.
  - RED: `env RUST_MIN_STACK=16777216 cargo test -p tau-gateway integration_spec_2838_c02_c03_ops_sessions_shell_renders_discovered_rows_and_chat_links -- --test-threads=1` failed while rows still rendered `/ops/chat?...session=...` links.
  - GREEN: `env RUST_MIN_STACK=16777216 cargo test -p tau-dashboard-ui functional_spec_2838_c01_c02_c03_sessions_route_renders_sessions_panel_list_rows_and_links -- --test-threads=1` passed.
  - GREEN: `env RUST_MIN_STACK=16777216 cargo test -p tau-gateway integration_spec_2838_c02_c03_ops_sessions_shell_renders_discovered_rows_and_chat_links -- --test-threads=1` passed.
  - GREEN: `env RUST_MIN_STACK=16777216 cargo test -p tau-dashboard-ui functional_spec_2838 -- --test-threads=1` passed (`2 passed`).
  - GREEN: `env RUST_MIN_STACK=16777216 cargo test -p tau-gateway functional_spec_2838 -- --test-threads=1` passed.
  - GREEN: `env RUST_MIN_STACK=16777216 cargo test -p tau-gateway integration_spec_2838 -- --test-threads=1` passed.
  - GREEN: `env RUST_MIN_STACK=16777216 cargo test -p tau-gateway spec_2842 -- --test-threads=1` passed (`2 passed`).
  - GREEN: `cargo fmt --package tau-dashboard-ui --package tau-gateway --check` passed.
  - GREEN: `env RUST_MIN_STACK=16777216 cargo clippy -p tau-dashboard-ui -p tau-gateway -- -D warnings` passed.
  - GREEN: `env RUST_MIN_STACK=16777216 cargo clippy -p tau-dashboard-ui -p tau-gateway --tests -- -D warnings` passed.
  - GREEN: `env RUST_MIN_STACK=16777216 cargo build -p tau-coding-agent` passed.
  - LIVE: restarted `tau-8795` from the rebuilt binary and loaded `/ops/sessions?theme=dark&sidebar=expanded&session=default`; Browser proof showed the `default` row link as `/ops/sessions/default?theme=dark&sidebar=expanded&session=default`, no stale `data-open-chat-session="default"` link, click navigated to `/ops/sessions/default?...`, detail panel visible with `data-session-key="default"`, timeline entry count `40`, and no browser console errors.
- Regression:
  - `cargo test -p tau-dashboard-ui functional_spec_2834 -- --test-threads=1`
  - `cargo test -p tau-gateway spec_2834 -- --test-threads=1`
- Verify:
  - `cargo fmt --check`
  - `cargo clippy -p tau-dashboard-ui -p tau-gateway -- -D warnings`
  - `python3 .github/scripts/oversized_file_guard.py`
  - `cargo mutants --in-diff /tmp/mutants_2838.diff -p tau-dashboard-ui -p tau-gateway` (`3/4` caught, `1` unviable compile-time mutant)
  - `cargo test -p tau-dashboard-ui`
  - `cargo test -p tau-gateway`
