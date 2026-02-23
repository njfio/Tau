# Spec: Issue #3392 - Cover CH15-05 lock-contention chaos scenario

Status: Implemented

## Problem Statement
`specs/3386/conformance-matrix.md` currently marks `CH15-05` as `N/A` even though gateway session persistence uses deterministic lock acquisition/retry behavior that can be exercised through existing `/v1/responses` integration paths.

## Scope
In scope:
- Add deterministic weekly-tier gateway chaos coverage for `CH15-05` (database/session lock contention).
- Assert lock contention is retried/queued successfully when the lock is released within configured wait policy.
- Assert session history remains intact after contention path execution.
- Update conformance mapping so `CH15-05` is no longer `N/A`.

Out of scope:
- New gateway API endpoints.
- Memory-pressure/GC scenario expansion for `CH15-06`.
- Provider fallback/circuit-breaker architecture changes.

## Acceptance Criteria
### AC-1 Lock contention is exercised in gateway E2E flow
Given an active gateway with deterministic lock settings and a pre-created session lock file,
when a `/v1/responses` request is issued for that session while the lock is briefly held and then released,
then the request succeeds without runtime crash and returns a valid response payload.

### AC-2 Session state remains valid after contention retry
Given the contention scenario above,
when session state is inspected after the request sequence,
then expected user/assistant lineage is preserved and no corruption/data loss is observed.

### AC-3 Conformance traceability is updated
Given issue conformance artifacts,
when reviewed,
then `CH15-05` is mapped to executable coverage instead of `N/A`.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
|---|---|---|---|---|---|
| C-01 | AC-1 | Chaos | pre-created session lock file + delayed lock release task | POST `/v1/responses` for locked session | request returns success and server remains healthy |
| C-02 | AC-2 | Chaos | session used in C-01 | inspect persisted session lineage after follow-up request | expected prompts/replies retained with no corruption |
| C-03 | AC-3 | Conformance | CH15 row in matrix | update mapping | `CH15-05` marked Covered with test reference |

## Success Metrics / Observable Signals
- `tier_weekly_ch15_chaos_matrix` deterministically covers `CH15-05`.
- `specs/3386/conformance-matrix.md` and `specs/3392/conformance-matrix.md` map `CH15-05` to executable coverage.
