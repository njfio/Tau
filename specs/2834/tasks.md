# Tasks: Issue #2834 - Chat active session selector contracts

## Ordered Tasks
1. [x] T1 (RED): add conformance tests for chat session selector SSR markers in `tau-dashboard-ui` and `/ops/chat` gateway rendering.
2. [x] T2 (GREEN): implement selector option snapshot support in UI and gateway session discovery.
3. [x] T3 (REGRESSION): run existing chat/send marker suites to ensure no contract regression.
4. [x] T4 (VERIFY): run fmt/clippy/tests/guardrails/mutation and set spec status to `Implemented`.
5. [x] T5 (RED/GREEN/REGRESSION): render the active chat session first so long historical selectors do not bury the current session.

## Tier Mapping
- Unit: session option normalization/discovery helper coverage (gateway).
- Property: N/A.
- Contract/DbC: N/A.
- Snapshot: N/A.
- Functional: UI `/ops/chat` selector markers.
- Conformance: C-01..C-03.
- Integration: gateway `/ops/chat` selection sync with seeded sessions.
- Fuzz: N/A.
- Mutation: `cargo mutants --in-diff <diff-file> -p tau-dashboard-ui -p tau-gateway`.
- Regression: existing ops chat shell suites (`functional_spec_2830`).
- Performance: N/A.

## Verification Evidence
- Targeted:
  - `cargo test -p tau-dashboard-ui functional_spec_2834 -- --test-threads=1`
  - `cargo test -p tau-gateway functional_spec_2834 -- --test-threads=1`
  - `cargo test -p tau-gateway integration_spec_2834 -- --test-threads=1`
  - RED: `env RUST_MIN_STACK=16777216 cargo test -p tau-dashboard-ui functional_spec_2834_c03_chat_route_adds_missing_active_session_option_marker -- --test-threads=1` failed before the missing active session rendered first.
  - RED: `env RUST_MIN_STACK=16777216 cargo test -p tau-gateway integration_spec_2834_c02_c03_ops_chat_selector_syncs_discovered_sessions_and_active_state -- --test-threads=1` failed before the selected discovered session rendered first.
  - GREEN: `env RUST_MIN_STACK=16777216 cargo test -p tau-dashboard-ui functional_spec_2834_c03_chat_route_adds_missing_active_session_option_marker -- --test-threads=1` passed.
  - GREEN: `env RUST_MIN_STACK=16777216 cargo test -p tau-gateway integration_spec_2834_c02_c03_ops_chat_selector_syncs_discovered_sessions_and_active_state -- --test-threads=1` passed.
  - VERIFY: `env RUST_MIN_STACK=16777216 cargo test -p tau-dashboard-ui -- --test-threads=1` passed (`213` tests, plus doc-tests).
  - VERIFY: `env RUST_MIN_STACK=16777216 cargo test -p tau-gateway -- --test-threads=1` passed (`370` tests, `1` ignored, plus doc-tests).
  - VERIFY: `env RUST_MIN_STACK=16777216 cargo clippy -p tau-dashboard-ui -p tau-gateway -- -D warnings` passed.
  - VERIFY: `env RUST_MIN_STACK=16777216 cargo clippy -p tau-gateway --tests -- -D warnings` passed.
  - VERIFY: `env RUST_MIN_STACK=16777216 cargo build -p tau-coding-agent` passed.
  - LIVE: restarted `127.0.0.1:8795`; `/ops/chat?...&session=default` rendered `tau-ops-chat-session-option-0` with `data-session-key="default"` and `data-selected="true"` before historical session keys.
- Regression:
  - `cargo test -p tau-dashboard-ui functional_spec_2830 -- --test-threads=1`
  - `cargo test -p tau-gateway functional_spec_2830 -- --test-threads=1`
- Verify:
  - `cargo fmt --check`
  - `cargo clippy -p tau-dashboard-ui -p tau-gateway -- -D warnings`
  - `python3 .github/scripts/oversized_file_guard.py`
  - `cargo mutants --in-diff /tmp/mutants_2834.diff -p tau-dashboard-ui -p tau-gateway` (`8/9` caught, `1` unviable compile-time mutant)
  - `cargo test -p tau-dashboard-ui`
  - `cargo test -p tau-gateway`
