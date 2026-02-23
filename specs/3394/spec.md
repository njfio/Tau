# Spec: Issue #3394 - Cover CH15-06 memory-pressure chaos scenario

Status: Accepted

## Problem Statement
`specs/3386/conformance-matrix.md` still marks `CH15-06` as `N/A`. We need deterministic gateway E2E coverage for high session-cardinality pressure (`100` sessions with history) so resilience under memory pressure is exercised by executable tests.

## Scope
In scope:
- Add deterministic weekly-tier gateway chaos coverage for `CH15-06`.
- Exercise high session-cardinality pressure by creating `100` unique sessions with multi-turn history.
- Assert gateway responsiveness and persisted session artifacts remain consistent during/after pressure run.
- Update conformance mapping so `CH15-06` is no longer `N/A`.

Out of scope:
- New gateway API surfaces.
- Direct process heap/RSS telemetry assertions (not exposed by stable gateway contract).
- Provider fallback/circuit-breaker architecture changes.

## Acceptance Criteria
### AC-1 High session-cardinality pressure executes without runtime failure
Given a deterministic weekly chaos harness,
when `100` session IDs each execute multi-turn `/v1/responses` traffic,
then requests complete successfully and no server panic/crash path is observed.

### AC-2 Session persistence and responsiveness remain valid post-pressure
Given the pressure run above,
when gateway session surfaces are inspected and a follow-up response is issued,
then the expected session set is discoverable, history artifacts are persisted, and follow-up traffic remains successful.

### AC-3 Conformance traceability is updated
Given issue conformance artifacts,
when reviewed,
then `CH15-06` is mapped to executable coverage instead of `N/A`.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
|---|---|---|---|---|---|
| C-01 | AC-1 | Chaos | weekly chaos harness with deterministic mock LLM | create 100 unique sessions with multi-turn requests | all pressure requests succeed without runtime failure |
| C-02 | AC-2 | Chaos | session population from C-01 | query sessions + inspect sample artifacts + send follow-up request | session persistence and gateway responsiveness remain valid |
| C-03 | AC-3 | Conformance | CH15 row in matrix | update mapping | `CH15-06` marked Covered with test reference |

## Success Metrics / Observable Signals
- `tier_weekly_ch15_chaos_matrix` deterministically covers `CH15-06`.
- `specs/3386/conformance-matrix.md` and `specs/3394/conformance-matrix.md` map `CH15-06` to executable coverage.
