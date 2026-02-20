# Spec: Issue #2893 - sessions list metadata contracts

Status: Implemented

## Problem Statement
The ops sessions list currently exposes route and selection contracts but does not expose deterministic row-level metadata contracts. This leaves the PRD checklist item “Session list shows all sessions with correct metadata” unverifiable from SSR output.

## Scope
In scope:
- Add deterministic sessions-list metadata markers to each discovered session row.
- Compute metadata from each session store/runtime file state.
- Verify metadata contracts in `tau-dashboard-ui` and `tau-gateway` conformance tests.

Out of scope:
- Session detail route behavior changes.
- Session branch/reset behavior changes.
- New dependencies.

## Acceptance Criteria
### AC-1 Sessions rows expose deterministic metadata markers
Given `/ops/sessions` renders discovered sessions,
when SSR markup is inspected,
then each row includes deterministic metadata markers for entry count, usage total tokens, validation status, and last-updated timestamp.

### AC-2 Metadata values are sourced from real session state
Given multiple sessions with different message/usage states,
when `/ops/sessions` renders,
then each row’s metadata markers reflect the corresponding session file’s runtime values.

### AC-3 Existing row contracts remain intact
Given sessions list row contracts already include session key + open-chat route,
when metadata is added,
then key, selected-state, and open-chat href contracts remain unchanged.

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
| C-01 | AC-1 | Functional | sessions route with discovered rows | inspect SSR markup | each row contains deterministic metadata markers |
| C-02 | AC-2 | Integration | sessions with distinct state/usage | render `/ops/sessions` | row marker values map to each session’s real metadata |
| C-03 | AC-3 | Regression | existing row route/selection contracts | render sessions route | session-key, selected-state, and open-chat href contracts persist |
| C-04 | AC-4 | Functional | no sessions discovered | render sessions route | empty-state marker renders with zero metadata rows |
| C-05 | AC-5 | Regression | prior suites | rerun suites | no regressions |

## Success Metrics / Signals
- `cargo test -p tau-dashboard-ui spec_2893 -- --test-threads=1` passes.
- `cargo test -p tau-gateway 'spec_2893' -- --test-threads=1` passes.
- Required chat/session regression suites remain green.
