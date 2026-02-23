# Spec: Issue #3400 - Cover F10 provider fallback/circuit-breaker scenario gaps

Status: Implemented

## Problem Statement
`specs/3386/conformance-matrix.md` still marks `F10-01`, `F10-02`, `F10-03`, `F10-04`, `F10-05`, and `F10-08` as `N/A`. The repository already contains deterministic provider fallback and circuit-breaker behavior in `tau-provider`, but conformance mapping is incomplete and key scenario assertions (`F10-01`, `F10-03`) need explicit executable coverage.

## Scope
In scope:
- Add/extend deterministic provider-routing tests for:
  - `F10-01` primary succeeds without invoking fallback.
  - `F10-03` all providers fail and chain exhaustion returns a deterministic error.
- Reuse and map existing deterministic tests for:
  - `F10-02` primary retryable failure falls back successfully.
  - `F10-04` circuit breaker opens and skips unhealthy route.
  - `F10-05` circuit breaker retries primary after cooldown.
  - `F10-08` fallback telemetry includes from/to route and failure metadata.
- Update conformance mappings so `F10-01/02/03/04/05/08` are no longer `N/A`.

Out of scope:
- Gateway API surface expansion.
- Rate-limit scenarios `F10-06` and `F10-07` (already covered).

## Acceptance Criteria
### AC-1 Primary success path does not fallback (`F10-01`)
Given an ordered route chain with a successful primary provider,
when completion executes,
then the primary response is returned and no fallback route is invoked.

### AC-2 Retryable failure and exhaustion behavior is deterministic (`F10-02`, `F10-03`)
Given ordered provider routes and retryable upstream failures,
when the primary fails and fallback succeeds,
then the fallback response is returned.
Given ordered provider routes where all routes fail retryably,
when completion executes,
then the call fails with deterministic chain-exhaustion behavior.

### AC-3 Circuit breaker open and half-open recovery are deterministic (`F10-04`, `F10-05`)
Given repeated retryable failures crossing configured threshold,
when subsequent requests execute before cooldown expiry,
then unhealthy routes are skipped.
Given cooldown elapsed,
when next request executes,
then the primary route is retried.

### AC-4 Fallback telemetry event contract is asserted (`F10-08`)
Given a retryable primary failure with successful fallback,
when fallback routing occurs,
then telemetry captures event type plus from/to model and error metadata.

### AC-5 Conformance traceability is updated
Given issue-local and milestone conformance artifacts,
when reviewed,
then `F10-01/02/03/04/05/08` map to executable tests instead of `N/A`.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
|---|---|---|---|---|---|
| C-01 | AC-1 | Functional | primary and secondary routes configured | primary returns success | primary response returned; secondary not called |
| C-02 | AC-2 | Functional | primary route returns retryable failure; secondary succeeds | completion executes | fallback response returned from secondary |
| C-03 | AC-2 | Functional | all routes return retryable failures | completion executes | deterministic error returned for exhausted chain |
| C-04 | AC-3 | Integration | circuit breaker threshold and cooldown configured | repeated failures cross threshold | open circuit skips unhealthy primary route |
| C-05 | AC-3 | Integration | primary circuit open and cooldown elapsed | next request executes | primary is retried and can recover |
| C-06 | AC-4 | Functional | fallback event sink configured | retryable primary failure triggers fallback | telemetry event contains type/from/to/status/error_kind |
| C-07 | AC-5 | Conformance | `F10-01/02/03/04/05/08` rows in matrix | conformance docs updated | rows marked `Covered` with executable tests |

## Success Metrics / Observable Signals
- Deterministic tests cover all in-scope `F10` scenarios.
- `specs/3386/conformance-matrix.md` and `specs/3400/conformance-matrix.md` map each in-scope `F10` scenario to executable tests.
