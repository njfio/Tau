# Tasks: Issue #2830 - Chat message send and transcript visibility contracts

## Ordered Tasks
1. [x] T1 (RED): add/extend conformance tests for chat send-form/transcript contracts in `tau-dashboard-ui` and `tau-gateway`.
2. [x] T2 (GREEN): implement gateway chat snapshot hydration + `POST /ops/chat/send` append/redirect behavior.
3. [x] T3 (REGRESSION): run targeted ops shell regression suites.
4. [x] T4 (VERIFY): run fmt/clippy/mutation/guardrails and set spec status to `Implemented`.
5. [x] T5 (REGRESSION): distinguish total session entries from rendered transcript rows and hidden system entries in the chat summary.
6. [x] T6 (RED/GREEN/REGRESSION): reject empty chat sends visibly via UI submit guard and backend `chat_status=empty-message` without mutating session state.

## Tier Mapping
- Unit: `ops_shell_controls` session query parsing unit tests.
- Property: N/A.
- Contract/DbC: N/A.
- Snapshot: N/A.
- Functional: UI `/ops/chat` marker assertions.
- Conformance: C-01..C-03, C-05..C-06.
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
  - LIVE: restarted `tau-8795` from the rebuilt binary with `--openai-auth-mode oauth-token --provider-subscription-strict=true`; browser empty-send click stayed on `/ops/chat`, marked form/input `data-submit-blocked="empty-message"`, and rendered `Message was not sent because it was empty.`
  - LIVE: browser non-empty send in session `ui-final-oauth-1778529948928` rendered two transcript rows and assistant text `OK` without `cortex_chat_llm_error_fallback`.
  - LIVE: backend whitespace-only POST redirected with `chat_status=empty-message` and left the generated session endpoint at `404`.
- Regression:
  - `cargo test -p tau-dashboard-ui functional_spec_2826 -- --test-threads=1`
  - `cargo test -p tau-gateway functional_spec_2802 -- --test-threads=1`
  - `cargo test -p tau-gateway functional_spec_2826 -- --test-threads=1`
- Verify:
  - `cargo fmt --check`
  - `cargo clippy -p tau-dashboard-ui -p tau-gateway -- -D warnings`
  - `python3 .github/scripts/oversized_file_guard.py`
  - `cargo mutants --in-diff /tmp/mutants_2830.diff -p tau-dashboard-ui -p tau-gateway` (`6/6` caught)
  - `cargo test -p tau-dashboard-ui`
  - `cargo test -p tau-gateway`
  - `cargo test` (workspace run; unrelated existing failures in `tau-coding-agent`, no failures in touched crates)
