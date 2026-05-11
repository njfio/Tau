# Tasks: Issue #2872 - chat new-session creation contracts

1. [x] T1 (RED): add failing `functional_spec_2872_*` UI tests for new-session form markers.
2. [x] T2 (RED): add failing `functional_spec_2872_*` + `integration_spec_2872_*` gateway tests for create+redirect+selector+hidden-route contracts.
3. [x] T3 (GREEN): implement additive UI new-session form markers and gateway `POST /ops/chat/new` behavior.
4. [x] T4 (REGRESSION): rerun `spec_2830`, `spec_2834`, `spec_2858`, `spec_2862`, `spec_2866`, and `spec_2870` suites.
5. [x] T5 (VERIFY): run fmt/clippy/scoped tests/mutation + fast live validation.
6. [x] T6 (RED/GREEN/REGRESSION): reject blank new-session submissions visibly with `new_session_status=empty-key` and no session store mutation.

## Verification Notes

- RED: `cargo test -p tau-dashboard-ui functional_spec_2872_c01_chat_route_renders_new_session_form_contract_markers -- --test-threads=1` failed before the hidden active-session field and empty-key guard existed.
- RED: `cargo test -p tau-gateway integration_spec_2872_c06_ops_chat_new_session_rejects_blank_key_with_visible_status -- --test-threads=1` failed before blank create preserved the active session with `new_session_status=empty-key`.
- GREEN: `env RUST_MIN_STACK=16777216 cargo test -p tau-dashboard-ui functional_spec_2872 -- --test-threads=1` passed.
- GREEN: `env RUST_MIN_STACK=16777216 cargo test -p tau-gateway spec_2872 -- --test-threads=1` passed.
- REGRESSION: `env RUST_MIN_STACK=16777216 cargo test -p tau-gateway spec_2830 -- --test-threads=1`, `spec_2834`, `spec_2858`, `spec_2862`, `spec_2866`, and `spec_2870` passed.
- VERIFY: `env RUST_MIN_STACK=16777216 cargo clippy -p tau-dashboard-ui -p tau-gateway -- -D warnings` passed.
- VERIFY: `env RUST_MIN_STACK=16777216 cargo clippy -p tau-gateway --tests -- -D warnings` passed.
- VERIFY: `env RUST_MIN_STACK=16777216 cargo build -p tau-coding-agent` passed.
- VERIFY: `env RUST_MIN_STACK=16777216 cargo test -p tau-dashboard-ui -- --test-threads=1` passed (`212` tests, plus doc-tests).
- VERIFY: `env RUST_MIN_STACK=16777216 cargo test -p tau-gateway -- --test-threads=1` passed (`370` tests, `1` ignored, plus doc-tests).
- LIVE: `POST /ops/chat/new` with a blank session key redirects to the active session with `new_session_status=empty-key` and does not create matching session state.
