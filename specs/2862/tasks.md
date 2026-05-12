# Tasks: Issue #2862 - chat token-counter marker contracts

1. [x] T1 (RED): add failing `functional_spec_2862_*` UI tests for chat token-counter marker presence/value contracts.
2. [x] T2 (RED): add failing `functional_spec_2862_*` and `integration_spec_2862_*` gateway tests for `/ops`, `/ops/chat`, `/ops/sessions` contracts.
3. [x] T3 (GREEN): implement chat token-counter marker attributes in `tau-dashboard-ui` chat panel markup.
4. [x] T4 (REGRESSION): rerun `spec_2830`, `spec_2842`, and `spec_2858` suites; fix any regressions.
5. [x] T5 (VERIFY): run fmt/clippy/scoped tests/mutation and fast live validation.
6. [x] T6 (REGRESSION): add visible assistant stream aggregates and keep non-chat route aggregates zeroed.
7. [x] T7 (RED/GREEN/REGRESSION): group token accounting details in a collapsed secondary token-counter manager.

## Verification Evidence
- RED: `env RUST_MIN_STACK=16777216 cargo test -p tau-dashboard-ui functional_spec_2862_c01_c02_c03_chat_route_renders_token_counter_marker_contract -- --test-threads=1` failed on `token-counter details manager should render`.
- RED: `env RUST_MIN_STACK=16777216 cargo test -p tau-gateway functional_spec_2862_c01_c02_c03_ops_chat_shell_exposes_token_counter_marker_contract -- --test-threads=1` failed on `token-counter details manager should render`.
- GREEN: `env RUST_MIN_STACK=16777216 cargo test -p tau-dashboard-ui functional_spec_2862_c01_c02_c03_chat_route_renders_token_counter_marker_contract -- --test-threads=1` passed.
- GREEN: `env RUST_MIN_STACK=16777216 cargo test -p tau-gateway functional_spec_2862_c01_c02_c03_ops_chat_shell_exposes_token_counter_marker_contract -- --test-threads=1` passed.
- GREEN: `env RUST_MIN_STACK=16777216 cargo test -p tau-dashboard-ui functional_spec_2862 -- --test-threads=1` passed (`2 passed`).
- GREEN: `env RUST_MIN_STACK=16777216 cargo test -p tau-gateway spec_2862 -- --test-threads=1` passed (`2 passed`).
- GREEN: `env RUST_MIN_STACK=16777216 cargo test -p tau-dashboard-ui -- --test-threads=1` passed (`215 passed`).
- GREEN: `env RUST_MIN_STACK=16777216 cargo test -p tau-gateway -- --test-threads=1` passed (`370 passed`, `1 ignored`).
- GREEN: `env RUST_MIN_STACK=16777216 cargo clippy -p tau-dashboard-ui -p tau-gateway -- -D warnings` passed.
- GREEN: `env RUST_MIN_STACK=16777216 cargo clippy -p tau-dashboard-ui --tests -- -D warnings` passed.
- GREEN: `env RUST_MIN_STACK=16777216 cargo clippy -p tau-gateway --tests -- -D warnings` passed.
- GREEN: `env RUST_MIN_STACK=16777216 cargo build -p tau-coding-agent` passed.
- LIVE: restarted `tau-8795` from the rebuilt binary and loaded `/ops/chat?theme=dark&sidebar=expanded&session=default`; Browser proof showed `Token counter: 0 total / 10 streams` with `data-collapsed-by-default="true"`, `tokenCounterVisibleBeforeOpen=false`, and `transcriptVisible=true`; expanding made the counter visible while the transcript stayed visible, and collapsing hid the counter again.
