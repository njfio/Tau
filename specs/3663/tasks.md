# Tasks: Issue #3663 - Persist gateway OpenResponses attempt payload traces for Ralph-loop debugging

- [x] T1 (RED): add a `tau-gateway` regression that proves a timed-out retry
      flow writes inspectable attempt-trace records with prompt/output/failure
      details.
- [x] T2 (GREEN): persist normalized gateway attempt trace records for success,
      continue, timeout, and runtime-failure paths.
- [x] T3 (VERIFY): run scoped `tau-gateway` retry/timeout regressions plus the
      new attempt-trace coverage.

## Tier Mapping
- Regression: persisted attempt-trace records for timed-out retries
- Functional: existing Ralph-loop retry success path remains green
