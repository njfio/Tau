# Spec: Issue #2893 - sessions list metadata contracts

Status: Implemented

## Problem Statement
The ops sessions list currently exposes route and selection contracts but does not expose deterministic row-level metadata contracts. This leaves the PRD checklist item “Session list shows all sessions with correct metadata” unverifiable from SSR output.

Follow-up live review found the metadata remained mostly contract-only: operators could see session names, but entry counts, token totals, validity, and last-update age were only available as hidden row attributes. The gateway metadata tests also matched broad `/ops/chat` hrefs from hidden chat controls instead of proving the sessions-row detail links.

## Scope
In scope:
- Add deterministic sessions-list metadata markers to each discovered session row.
- Render the same metadata visibly in each sessions-row card.
- Compute metadata from each session store/runtime file state.
- Verify metadata contracts in `tau-dashboard-ui` and `tau-gateway` conformance tests.

Out of scope:
- Session detail route behavior changes.
- Session branch/reset behavior changes.
- New dependencies.

## Acceptance Criteria
### AC-1 Sessions rows expose visible deterministic metadata
Given `/ops/sessions` renders discovered sessions,
when SSR markup is inspected,
then each row includes visible metadata plus deterministic markers for entry count, usage total tokens, validation status, and last-updated timestamp.

### AC-2 Metadata values are sourced from real session state
Given multiple sessions with different message/usage states,
when `/ops/sessions` renders,
then each row’s metadata markers reflect the corresponding session file’s runtime values, including system, user, and assistant entries persisted by completed chat turns.

### AC-3 Existing row contracts remain detail-route specific
Given sessions list row contracts already include session key + session-detail route,
when metadata is added,
then key, selected-state, and `/ops/sessions/{session_key}` href contracts remain unchanged and tests do not pass by matching hidden `/ops/chat` controls.

### AC-4 Empty-state behavior is unchanged
Given no discovered sessions,
when `/ops/sessions` renders,
then empty-state markers still render and no phantom metadata rows appear.

### AC-5 Regression safety for prior chat/session phases
Given existing suites for `spec_2830`, `spec_2834`, `spec_2838`, `spec_2842`, `spec_2846`, and `spec_2885`,
when rerun,
then all suites remain green.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
|---|---|---|---|---|---|
| C-01 | AC-1 | Functional | sessions route with discovered rows | inspect SSR markup | each row contains visible deterministic metadata markers |
| C-02 | AC-2 | Integration | sessions with distinct state/usage | render `/ops/sessions` | row marker values map to each session’s real metadata |
| C-03 | AC-3 | Regression | existing row route/selection contracts | render sessions route | session-key, selected-state, and session-detail href contracts persist |
| C-04 | AC-4 | Functional | no sessions discovered | render sessions route | empty-state marker renders with zero metadata rows |
| C-05 | AC-5 | Regression | prior suites | rerun suites | no regressions |

## Success Metrics / Signals
- `cargo test -p tau-dashboard-ui spec_2893 -- --test-threads=1` passes.
- `cargo test -p tau-gateway 'spec_2893' -- --test-threads=1` passes.
- Required chat/session regression suites remain green.
