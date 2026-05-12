# Plan: Issue #2872 - chat new-session creation contracts

## Approach
1. Add additive chat new-session form markers in `tau-dashboard-ui` chat panel.
2. Add `POST /ops/chat/new` gateway handler that initializes target session and redirects to chat route preserving theme/sidebar query state.
3. Add UI and gateway conformance tests for create+redirect+selector+hidden-route panel contracts.
4. Add an empty-key submit guard and backend `new_session_status=empty-key` redirect path that preserves the active session without initializing new session state.
5. Add a successful-create redirect/status path with `new_session_status=created` so the selected empty session is visibly acknowledged.
6. Re-run required chat regressions and verification gates.

## Affected Modules
- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-gateway/src/gateway_openresponses.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`

## Risks and Mitigations
- Risk: route/query-state regressions across chat/sessions views.
  - Mitigation: integration tests for `/ops/chat`, `/ops`, and `/ops/sessions` after new-session creation.
- Risk: session selector order assumptions in tests.
  - Mitigation: assert presence/selection markers, not fragile ordering beyond deterministic contract expectations.
- Risk: blank new-session submission silently redirects to `default` and looks successful.
  - Mitigation: submit guard plus backend status marker preserves active session context and explains the rejection.
- Risk: valid new-session submission creates an empty selected session that looks like an inert control, especially when a long session list pushes the result out of the first viewport.
  - Mitigation: integration/UI tests for `new_session_status=created`, visible success copy, and create/status placement before the historical session selector.
- Risk: created or rejected new-session status is present in markup but hidden behind the collapsed session manager.
  - Mitigation: open/expose the session manager on first render for non-idle new-session status while preserving collapsed-by-default behavior for idle chat loads.

## Interface / Contract Notes
- Additive route `POST /ops/chat/new` (internal ops shell endpoint).
- Blank new-session submissions redirect to `/ops/chat?...&session=<active>&new_session_status=empty-key`.
- Valid new-session submissions redirect to `/ops/chat?...&session=<created>&new_session_status=created`.
- `/ops/chat` renders visible new-session status near the create form for both rejected and successful create outcomes.
- `id="tau-ops-chat-session-manager"` exposes `data-session-manager-initial-open="true"` for non-idle new-session results so status content is not hidden after redirect.
- No schema/protocol changes outside ops shell route handling.
