# Tasks: Issue #2862 - chat token-counter marker contracts

1. [x] T1 (RED): add failing `functional_spec_2862_*` UI tests for chat token-counter marker presence/value contracts.
2. [x] T2 (RED): add failing `functional_spec_2862_*` and `integration_spec_2862_*` gateway tests for `/ops`, `/ops/chat`, `/ops/sessions` contracts.
3. [x] T3 (GREEN): implement chat token-counter marker attributes in `tau-dashboard-ui` chat panel markup.
4. [x] T4 (REGRESSION): rerun `spec_2830`, `spec_2842`, and `spec_2858` suites; fix any regressions.
5. [x] T5 (VERIFY): run fmt/clippy/scoped tests/mutation and fast live validation.
6. [x] T6 (REGRESSION): add visible assistant stream aggregates and keep non-chat route aggregates zeroed.
7. [x] T7 (RED/GREEN/REGRESSION): group token accounting details in a collapsed secondary token-counter manager.
8. [x] T8 (RED/GREEN/REGRESSION): use rendered assistant stream-token totals in the collapsed summary when provider usage totals are unavailable.
9. [x] T9 (RED/GREEN/REGRESSION): sync collapsed token-counter detail accessibility state with the token-counter manager toggle.

## Verification Evidence
- RED: `env RUST_MIN_STACK=16777216 cargo test -p tau-dashboard-ui functional_spec_2862_c01_c02_c03_chat_route_renders_token_counter_marker_contract -- --test-threads=1` failed on `token-counter details manager should render`.
- RED: `env RUST_MIN_STACK=16777216 cargo test -p tau-gateway functional_spec_2862_c01_c02_c03_ops_chat_shell_exposes_token_counter_marker_contract -- --test-threads=1` failed on `token-counter details manager should render`.
- GREEN: `env RUST_MIN_STACK=16777216 cargo test -p tau-dashboard-ui functional_spec_2862_c01_c02_c03_chat_route_renders_token_counter_marker_contract -- --test-threads=1` passed.
- GREEN: `env RUST_MIN_STACK=16777216 cargo test -p tau-gateway functional_spec_2862_c01_c02_c03_ops_chat_shell_exposes_token_counter_marker_contract -- --test-threads=1` passed.
- GREEN: `env RUST_MIN_STACK=16777216 cargo test -p tau-dashboard-ui functional_spec_2862 -- --test-threads=1` passed (`3 passed`), including C-08 stream-token summary fallback.
- GREEN: `env RUST_MIN_STACK=16777216 cargo test -p tau-gateway spec_2862 -- --test-threads=1` passed (`2 passed`).
- RED: `env RUST_MIN_STACK=16777216 cargo test -p tau-dashboard-ui functional_spec_2862_c01_c02_c03_chat_route_renders_token_counter_marker_contract -- --test-threads=1` failed on missing `aria-hidden="true" data-token-counter-details-open="false"`.
- RED: `env RUST_MIN_STACK=16777216 cargo test -p tau-gateway functional_spec_2862_c01_c02_c03_ops_chat_shell_exposes_token_counter_marker_contract -- --test-threads=1` failed on missing `aria-hidden="true" data-token-counter-details-open="false"`.
- GREEN: `env RUST_MIN_STACK=16777216 cargo test -p tau-dashboard-ui functional_spec_2862_c01_c02_c03_chat_route_renders_token_counter_marker_contract -- --test-threads=1` passed.
- GREEN: `env RUST_MIN_STACK=16777216 cargo test -p tau-gateway functional_spec_2862_c01_c02_c03_ops_chat_shell_exposes_token_counter_marker_contract -- --test-threads=1` passed.
- GREEN: `env RUST_MIN_STACK=16777216 cargo test -p tau-dashboard-ui functional_spec_2862 -- --test-threads=1` passed (`3 passed`) with C-09 token-counter visibility sync assertions.
- GREEN: `env RUST_MIN_STACK=16777216 cargo test -p tau-gateway spec_2862 -- --test-threads=1` passed (`2 passed`) with C-09 token-counter visibility sync assertions.
- GREEN: `cargo fmt --package tau-dashboard-ui --package tau-gateway --check` passed.
- GREEN: `env RUST_MIN_STACK=16777216 cargo clippy -p tau-dashboard-ui -p tau-gateway -- -D warnings` passed.
- GREEN: `env RUST_MIN_STACK=16777216 cargo clippy -p tau-dashboard-ui -p tau-gateway --tests -- -D warnings` passed.
- GREEN: `env RUST_MIN_STACK=16777216 cargo build -p tau-coding-agent` passed.
- LIVE: restarted `tau-8795` from the rebuilt binary and loaded `/ops/chat?theme=dark&sidebar=expanded&session=default`; Browser proof showed the collapsed token counter starts `aria-hidden="true"` / `data-token-counter-details-open="false"`, opens to `aria-hidden="false"` / `data-token-counter-details-open="true"`, closes back to hidden, keeps `Token counter: 651 stream tokens / 15 streams`, and has no browser console errors.
- GREEN: `env RUST_MIN_STACK=16777216 cargo test -p tau-dashboard-ui functional_spec_2901_c01_c03_chat_route_renders_assistant_token_stream_markers -- --test-threads=1` passed.
- GREEN: `env RUST_MIN_STACK=16777216 cargo test -p tau-gateway integration_spec_2901_c01_c02_c03_ops_chat_renders_assistant_token_stream_markers_in_order -- --test-threads=1` passed.
- GREEN: `env RUST_MIN_STACK=16777216 cargo test -p tau-dashboard-ui functional_spec_2830 -- --test-threads=1` passed (`5 passed`).
- GREEN: `env RUST_MIN_STACK=16777216 cargo test -p tau-gateway functional_spec_2830 -- --test-threads=1` passed (`1 passed`).
- GREEN: `env RUST_MIN_STACK=16777216 cargo test -p tau-gateway integration_spec_2830 -- --test-threads=1` passed (`2 passed`).
- GREEN: `env RUST_MIN_STACK=16777216 cargo test -p tau-dashboard-ui -- --test-threads=1` passed (`215 passed`).
- GREEN: `env RUST_MIN_STACK=16777216 cargo test -p tau-gateway -- --test-threads=1` passed (`370 passed`, `1 ignored`).
- GREEN: `env RUST_MIN_STACK=16777216 cargo clippy -p tau-dashboard-ui -p tau-gateway -- -D warnings` passed.
- GREEN: `env RUST_MIN_STACK=16777216 cargo clippy -p tau-dashboard-ui -p tau-gateway --tests -- -D warnings` passed.
- GREEN: `env RUST_MIN_STACK=16777216 cargo build -p tau-coding-agent` passed.
- LIVE: restarted `tau-8795` from the rebuilt binary and loaded `/ops/chat?theme=dark&sidebar=expanded&session=default`; Browser proof showed `Token counter: 651 stream tokens / 15 streams`, `data-summary-token-source="assistant-stream"`, `data-summary-tokens="651"`, provider `data-total-tokens="0"`, and no browser console errors.
