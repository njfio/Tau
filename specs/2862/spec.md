# Spec: Issue #2862 - chat token-counter marker contracts

Status: Reviewed

## Problem Statement
Tau Ops chat currently renders transcript and session selector contracts but lacks an explicit token-counter marker contract tied to active-session usage summary fields. This blocks deterministic selector-based validation for the PRD requirement that token counters update with conversation state.

## Scope
In scope:
- Add deterministic chat token-counter marker attributes to the chat panel output.
- Bind marker values to active-session usage snapshot fields (`input`, `output`, `total` tokens).
- Validate route-safe behavior on `/ops`, `/ops/chat`, and `/ops/sessions`.

Out of scope:
- Streaming token transport changes.
- Session usage accounting algorithm changes.
- New endpoints or schema changes.

## Acceptance Criteria
### AC-1 Chat token-counter marker exists with deterministic attributes
Given any ops shell render,
when chat panel markup is rendered,
then it includes deterministic `tau-ops-chat-token-counter` marker attributes for session key and token totals.

### AC-2 Marker values mirror active-session usage summary
Given chat snapshot usage values,
when shell renders,
then marker attributes match `input/output/total` usage values exactly.

### AC-3 `/ops/chat` route exposes visible token-counter marker contract
Given `/ops/chat` shell request,
when response renders,
then chat token-counter marker is present with deterministic attributes and chat panel remains visible.

### AC-4 Non-chat routes preserve hidden-panel token-counter contract
Given `/ops` and `/ops/sessions` shell requests,
when response renders,
then chat panel remains hidden but still includes deterministic token-counter marker attributes.

### AC-5 Regression safety for existing chat/session/detail contracts
Given prior contract suites,
when `spec_2830`, `spec_2842`, and `spec_2858` rerun,
then all existing route/panel/session-detail contracts remain green.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
|---|---|---|---|---|---|
| C-01 | AC-1 | Functional | ops shell render | render UI shell | `tau-ops-chat-token-counter` marker exists with deterministic attributes |
| C-02 | AC-2 | Functional | non-zero usage snapshot fields | render UI shell | marker `data-input-tokens`, `data-output-tokens`, `data-total-tokens` match snapshot values |
| C-03 | AC-3 | Integration | gateway `/ops/chat` request | render response | chat panel visible and token-counter marker attributes present |
| C-04 | AC-4 | Integration | gateway `/ops` + `/ops/sessions` requests | render response | chat panel hidden and token-counter marker attributes still present |
| C-05 | AC-5 | Regression | existing contract suites | rerun suites | no regressions in existing chat/session/detail visibility contracts |

## Success Metrics / Signals
- `cargo test -p tau-dashboard-ui spec_2862 -- --test-threads=1` passes.
- `cargo test -p tau-gateway 'spec_2862' -- --test-threads=1` passes.
- `spec_2830`, `spec_2842`, and `spec_2858` suites remain green.
