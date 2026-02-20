# Spec: Issue #2858 - chat/sessions panel visibility-state contracts

Status: Reviewed

## Problem Statement
Tau Ops chat and sessions panels already expose route and `aria-hidden` contracts, but they do not expose explicit panel visibility-state attributes for deterministic selector-based validation. Downstream validation harnesses currently infer state indirectly from `aria-hidden` only.

## Scope
In scope:
- Add deterministic `data-panel-visible` marker attributes to chat and sessions panels.
- Validate visibility-state combinations for `/ops`, `/ops/chat`, and `/ops/sessions`.
- Preserve existing route and panel visibility behavior.

Out of scope:
- Navigation or route model changes.
- New API endpoints or payload schema updates.
- Command-center behavior changes.

## Acceptance Criteria
### AC-1 Chat panel exposes explicit visibility state
Given any ops route render,
when chat panel is rendered,
then panel marker includes deterministic `data-panel-visible` matching route visibility.

### AC-2 Sessions panel exposes explicit visibility state
Given any ops route render,
when sessions panel is rendered,
then panel marker includes deterministic `data-panel-visible` matching route visibility.

### AC-3 `/ops/chat` state combination
Given request to `/ops/chat`,
when shell renders,
then chat panel has `data-panel-visible="true"` and sessions panel has `data-panel-visible="false"`.

### AC-4 `/ops/sessions` state combination
Given request to `/ops/sessions`,
when shell renders,
then sessions panel has `data-panel-visible="true"` and chat panel has `data-panel-visible="false"`.

### AC-5 `/ops` baseline hidden combination + regression safety
Given request to `/ops`,
when shell renders,
then both chat and sessions panels have `data-panel-visible="false"`, and existing route/panel contracts remain unchanged.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
|---|---|---|---|---|---|
| C-01 | AC-1 | Functional | `/ops/chat` route render | shell render | chat panel includes `data-panel-visible="true"` |
| C-02 | AC-2 | Functional | `/ops/sessions` route render | shell render | sessions panel includes `data-panel-visible="true"` |
| C-03 | AC-3 | Integration | gateway `/ops/chat` request | render response | chat=true and sessions=false |
| C-04 | AC-4 | Integration | gateway `/ops/sessions` request | render response | sessions=true and chat=false |
| C-05 | AC-5 | Regression | gateway `/ops` request + existing suites | render and rerun | chat=false + sessions=false and prior contracts remain green |

## Success Metrics / Signals
- `cargo test -p tau-dashboard-ui functional_spec_2858 -- --test-threads=1` passes.
- `cargo test -p tau-gateway 'spec_2858' -- --test-threads=1` passes.
- Regression suites for related contracts (`spec_2830`, `spec_2838`, `spec_2854`) remain green.
