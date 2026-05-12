# Tasks: Issue #2830 - Chat message send and transcript visibility contracts

## Ordered Tasks
1. [x] T1 (RED): add/extend conformance tests for chat send-form/transcript contracts in `tau-dashboard-ui` and `tau-gateway`.
2. [x] T2 (GREEN): implement gateway chat snapshot hydration + `POST /ops/chat/send` append/redirect behavior.
3. [x] T3 (REGRESSION): run targeted ops shell regression suites.
4. [x] T4 (VERIFY): run fmt/clippy/mutation/guardrails and set spec status to `Implemented`.
5. [x] T5 (REGRESSION): distinguish total session entries from rendered transcript rows and hidden system entries in the chat summary.
6. [x] T6 (RED/GREEN/REGRESSION): reject empty chat sends visibly via UI submit guard and backend `chat_status=empty-message` without mutating session state.
7. [x] T7 (RED/GREEN/REGRESSION): keep active compose controls before historical session selection on `/ops/chat`.
8. [x] T8 (RED/GREEN/REGRESSION): keep active compose controls before secondary new-session creation on `/ops/chat`.
9. [x] T9 (RED/GREEN/REGRESSION): group secondary new-session and session-history controls in a compact collapsed session manager.
10. [x] T10 (RED/GREEN/REGRESSION): keep active compose controls before session metadata and navigation links on `/ops/chat`.
11. [x] T11 (RED/GREEN/REGRESSION): group secondary session metadata and navigation links in a compact collapsed session-details manager.
12. [x] T12 (RED/GREEN/REGRESSION): group verbose latest-turn proof in a compact collapsed latest-turn manager.

## Tier Mapping
- Unit: `ops_shell_controls` session query parsing unit tests.
- Property: N/A.
- Contract/DbC: N/A.
- Snapshot: N/A.
- Functional: UI `/ops/chat` marker assertions.
- Conformance: C-01..C-03, C-05..C-12.
- Integration: gateway send + redirect + transcript visibility assertions.
- Fuzz: N/A.
- Mutation: `cargo mutants --in-diff <diff-file> -p tau-dashboard-ui -p tau-gateway`.
- Regression: targeted existing ops-shell suites.
- Performance: N/A.

## Verification Evidence
- Targeted:
  - `cargo test -p tau-dashboard-ui functional_spec_2830 -- --test-threads=1`
  - `cargo test -p tau-gateway functional_spec_2830 -- --test-threads=1`
  - `cargo test -p tau-gateway integration_spec_2830 -- --test-threads=1`
  - `cargo test -p tau-gateway unit_requested_session_key -- --test-threads=1`
  - `cargo test -p tau-dashboard-ui functional_spec_2830_c01_chat_route_renders_send_form_and_fallback_transcript_markers -- --test-threads=1`
  - `cargo test -p tau-gateway integration_spec_2830_c06_ops_chat_send_rejects_empty_message_with_visible_status -- --test-threads=1`
  - RED: `cargo test -p tau-dashboard-ui functional_spec_2830_c07_chat_route_prioritizes_composer_before_session_selector -- --test-threads=1` failed on `chat composer should render before historical session selector`.
  - RED: `cargo test -p tau-gateway functional_spec_2830_c01_ops_chat_shell_exposes_send_form_and_fallback_transcript_markers -- --test-threads=1` failed on `chat composer should render before historical session selector`.
  - GREEN: `cargo test -p tau-dashboard-ui functional_spec_2830_c07_chat_route_prioritizes_composer_before_session_selector -- --test-threads=1` passed.
  - GREEN: `cargo test -p tau-gateway functional_spec_2830_c01_ops_chat_shell_exposes_send_form_and_fallback_transcript_markers -- --test-threads=1` passed.
  - GREEN: `cargo test -p tau-dashboard-ui functional_spec_2830 -- --test-threads=1` passed (`4 passed`).
  - GREEN: `cargo test -p tau-gateway functional_spec_2830 -- --test-threads=1` passed.
  - GREEN: `cargo test -p tau-gateway integration_spec_2830 -- --test-threads=1` passed (`2 passed`).
  - LIVE: restarted `tau-8795` from the rebuilt binary with `--openai-auth-mode oauth-token --provider-subscription-strict=true`; browser empty-send click stayed on `/ops/chat`, marked form/input `data-submit-blocked="empty-message"`, and rendered `Message was not sent because it was empty.`
  - LIVE: browser non-empty send in session `ui-final-oauth-1778529948928` rendered two transcript rows and assistant text `OK` without `cortex_chat_llm_error_fallback`.
  - LIVE: backend whitespace-only POST redirected with `chat_status=empty-message` and left the generated session endpoint at `404`.
  - LIVE: restarted `tau-8795` from the rebuilt binary and loaded `/ops/chat?theme=dark&sidebar=expanded&session=default`; browser visible DOM showed `textarea` + `Send` before historical session links.
  - LIVE: `POST /ops/chat/send` smoke for `ui-compose-order-curl-1778538967` returned `303 See Other` to `/ops/chat?...session=ui-compose-order-curl-1778538967`; reloaded browser route showed current compose controls before the active session link and the served HTML indexes were `send-form=57144`, `send-status=63393`, `session-selector=63572`, `transcript=69938`, submitted message=68656.
  - RED: `cargo test -p tau-dashboard-ui functional_spec_2830_c07_chat_route_prioritizes_composer_before_session_selector -- --test-threads=1` failed on `chat composer should render before secondary new-session creation`.
  - RED: `cargo test -p tau-gateway functional_spec_2830_c01_ops_chat_shell_exposes_send_form_and_fallback_transcript_markers -- --test-threads=1` failed on `chat composer should render before secondary new-session creation`.
  - GREEN: `cargo test -p tau-dashboard-ui functional_spec_2830_c07_chat_route_prioritizes_composer_before_session_selector -- --test-threads=1` passed.
  - GREEN: `cargo test -p tau-gateway functional_spec_2830_c01_ops_chat_shell_exposes_send_form_and_fallback_transcript_markers -- --test-threads=1` passed.
  - GREEN: `cargo test -p tau-dashboard-ui functional_spec_2830 -- --test-threads=1` passed (`4 passed`).
  - GREEN: `cargo test -p tau-gateway functional_spec_2830 -- --test-threads=1` passed.
  - GREEN: `cargo test -p tau-gateway integration_spec_2830 -- --test-threads=1` passed (`2 passed`).
  - LIVE: restarted `tau-8795` from the rebuilt binary and loaded `/ops/chat?theme=dark&sidebar=expanded&session=default`; browser visible DOM indexes were `textarea=2485`, `Send=2610`, `Create Session=2717`, first session link=`2827`, proving active compose before new-session and history controls.
  - RED: `env RUST_MIN_STACK=16777216 cargo test -p tau-dashboard-ui functional_spec_2830_c07_chat_route_prioritizes_composer_before_session_selector -- --test-threads=1` failed on `session manager marker should render`.
  - RED: `env RUST_MIN_STACK=16777216 cargo test -p tau-gateway functional_spec_2830_c01_ops_chat_shell_exposes_send_form_and_fallback_transcript_markers -- --test-threads=1` failed on `session manager marker should render`.
  - GREEN: `env RUST_MIN_STACK=16777216 cargo test -p tau-dashboard-ui functional_spec_2830_c07_chat_route_prioritizes_composer_before_session_selector -- --test-threads=1` passed.
  - GREEN: `env RUST_MIN_STACK=16777216 cargo test -p tau-gateway functional_spec_2830_c01_ops_chat_shell_exposes_send_form_and_fallback_transcript_markers -- --test-threads=1` passed.
  - GREEN: `env RUST_MIN_STACK=16777216 cargo test -p tau-dashboard-ui functional_spec_2830 -- --test-threads=1` passed (`4 passed`).
  - GREEN: `env RUST_MIN_STACK=16777216 cargo test -p tau-gateway functional_spec_2830 -- --test-threads=1` passed.
  - GREEN: `env RUST_MIN_STACK=16777216 cargo test -p tau-gateway integration_spec_2830 -- --test-threads=1` passed (`2 passed`).
  - LIVE: restarted `tau-8795` from the rebuilt binary with OAuth-backed launch flags and loaded `/ops/chat?theme=dark&sidebar=expanded&session=default`; Browser proof showed collapsed default state `summaryText="Sessions (16)"`, `formVisible=false`, `selectorVisible=false`, `visibleLinks=0`, then after clicking the manager `formVisible=true`, `selectorVisible=true`, `visibleLinks=16`.
  - RED: `env RUST_MIN_STACK=16777216 cargo test -p tau-dashboard-ui functional_spec_2830_c07_chat_route_prioritizes_composer_before_session_selector -- --test-threads=1` failed on `chat composer should be the first active chat control before session metadata`.
  - GREEN: `env RUST_MIN_STACK=16777216 cargo test -p tau-dashboard-ui functional_spec_2830_c07_chat_route_prioritizes_composer_before_session_selector -- --test-threads=1` passed.
  - GREEN: `env RUST_MIN_STACK=16777216 cargo test -p tau-gateway functional_spec_2830_c01_ops_chat_shell_exposes_send_form_and_fallback_transcript_markers -- --test-threads=1` passed.
  - GREEN: `env RUST_MIN_STACK=16777216 cargo test -p tau-dashboard-ui functional_spec_2830 -- --test-threads=1` passed (`4 passed`).
  - GREEN: `env RUST_MIN_STACK=16777216 cargo test -p tau-gateway functional_spec_2830 -- --test-threads=1` passed.
  - GREEN: `env RUST_MIN_STACK=16777216 cargo test -p tau-gateway integration_spec_2830 -- --test-threads=1` passed (`2 passed`).
  - LIVE: restarted `tau-8795` from the rebuilt binary and loaded `/ops/chat?theme=dark&sidebar=expanded&session=default`; Browser proof showed `textarea=1763`, `Send=1940`, `Open Session Detail=2054`, `Jump To Latest=2132`, and `Sessions (16)=2175`, with the session manager still collapsed (`newSessionFormVisible=false`, `selectorVisible=false`).
  - RED: `env RUST_MIN_STACK=16777216 cargo test -p tau-dashboard-ui functional_spec_2830_c07_chat_route_prioritizes_composer_before_session_selector -- --test-threads=1` failed on `session details manager should render`.
  - GREEN: `env RUST_MIN_STACK=16777216 cargo test -p tau-dashboard-ui functional_spec_2830_c07_chat_route_prioritizes_composer_before_session_selector -- --test-threads=1` passed.
  - GREEN: `env RUST_MIN_STACK=16777216 cargo test -p tau-gateway functional_spec_2830_c01_ops_chat_shell_exposes_send_form_and_fallback_transcript_markers -- --test-threads=1` passed.
  - GREEN: `env RUST_MIN_STACK=16777216 cargo test -p tau-dashboard-ui functional_spec_2830 -- --test-threads=1` passed (`4 passed`).
  - GREEN: `env RUST_MIN_STACK=16777216 cargo test -p tau-gateway functional_spec_2830 -- --test-threads=1` passed.
  - GREEN: `env RUST_MIN_STACK=16777216 cargo test -p tau-gateway integration_spec_2830 -- --test-threads=1` passed (`2 passed`).
  - LIVE: restarted `tau-8795` from the rebuilt binary and loaded `/ops/chat?theme=dark&sidebar=expanded&session=default`; Browser proof showed `Session: default (30 entries)` collapsed with `sessionSummaryVisible=false` and `sessionActionsVisible=false`, expanding it made both visible, and collapsing it hid both again while `Sessions (16)` stayed collapsed.
  - RED: `env RUST_MIN_STACK=16777216 cargo test -p tau-dashboard-ui functional_spec_2830_c02_chat_route_renders_snapshot_message_rows_for_active_session -- --test-threads=1` failed on `latest-turn details manager should render`.
  - RED: `env RUST_MIN_STACK=16777216 cargo test -p tau-gateway functional_spec_2830_c01_ops_chat_shell_exposes_send_form_and_fallback_transcript_markers -- --test-threads=1` failed on missing `tau-ops-chat-latest-turn-details`.
  - GREEN: `env RUST_MIN_STACK=16777216 cargo test -p tau-dashboard-ui functional_spec_2830_c02_chat_route_renders_snapshot_message_rows_for_active_session -- --test-threads=1` passed.
  - GREEN: `env RUST_MIN_STACK=16777216 cargo test -p tau-gateway functional_spec_2830_c01_ops_chat_shell_exposes_send_form_and_fallback_transcript_markers -- --test-threads=1` passed.
  - GREEN: `env RUST_MIN_STACK=16777216 cargo test -p tau-dashboard-ui functional_spec_2830 -- --test-threads=1` passed (`4 passed`).
  - GREEN: `env RUST_MIN_STACK=16777216 cargo test -p tau-gateway functional_spec_2830 -- --test-threads=1` passed.
  - GREEN: `env RUST_MIN_STACK=16777216 cargo test -p tau-gateway integration_spec_2830 -- --test-threads=1` passed (`2 passed`).
  - LIVE: restarted `tau-8795` from the rebuilt binary and loaded `/ops/chat?theme=dark&sidebar=expanded&session=default`; Browser proof showed `Latest turn: user 27 / assistant 28` collapsed by default with `latestArticleVisibleBeforeOpen=false` and `transcriptVisible=true`, expanding it made the latest-turn article visible while the transcript stayed visible, and collapsing it hid the article again.
- Regression:
  - `cargo test -p tau-dashboard-ui functional_spec_2826 -- --test-threads=1`
  - `cargo test -p tau-gateway functional_spec_2802 -- --test-threads=1`
  - `cargo test -p tau-gateway functional_spec_2826 -- --test-threads=1`
- Verify:
  - `cargo fmt --check`
  - `cargo fmt --package tau-dashboard-ui --package tau-gateway --check`
  - `git diff --check`
  - `cargo clippy -p tau-dashboard-ui -p tau-gateway -- -D warnings`
  - `cargo clippy -p tau-dashboard-ui --tests -- -D warnings`
  - `cargo clippy -p tau-gateway --tests -- -D warnings`
  - `env RUST_MIN_STACK=16777216 cargo build -p tau-coding-agent`
  - `env RUST_MIN_STACK=16777216 cargo test -p tau-dashboard-ui -- --test-threads=1` (`214 passed`)
  - `env RUST_MIN_STACK=16777216 cargo test -p tau-gateway -- --test-threads=1` (`370 passed`, `1 ignored`)
  - `env RUST_MIN_STACK=16777216 cargo clippy -p tau-dashboard-ui -p tau-gateway -- -D warnings`
  - `env RUST_MIN_STACK=16777216 cargo clippy -p tau-dashboard-ui --tests -- -D warnings`
  - `env RUST_MIN_STACK=16777216 cargo clippy -p tau-gateway --tests -- -D warnings`
  - `env RUST_MIN_STACK=16777216 cargo build -p tau-coding-agent`
  - `python3 .github/scripts/oversized_file_guard.py`
  - `cargo mutants --in-diff /tmp/mutants_2830.diff -p tau-dashboard-ui -p tau-gateway` (`6/6` caught)
  - `cargo test -p tau-dashboard-ui`
  - `cargo test -p tau-gateway`
  - `cargo test` (workspace run; unrelated existing failures in `tau-coding-agent`, no failures in touched crates)
