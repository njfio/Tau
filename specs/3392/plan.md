# Plan: Issue #3392

## Approach
1. Extend `tier_weekly_ch15_chaos_matrix` with a deterministic `CH15-05` path that:
   - pre-creates the target session lock file,
   - releases it after a short delay in a background task,
   - sends `/v1/responses` for the locked session and asserts success.
2. Verify persisted session lineage after contention and a follow-up request to ensure no data loss/corruption.
3. Update conformance mapping artifacts for `CH15-05`.
4. Run verification gates for touched scope.

## Affected Modules
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`
- `specs/3386/conformance-matrix.md`
- `specs/3392/spec.md`
- `specs/3392/plan.md`
- `specs/3392/tasks.md`
- `specs/3392/conformance-matrix.md`
- `specs/milestones/m291/index.md`

## Risks and Mitigations
- Risk: lock contention assertions become timing-flaky.
  Mitigation: use deterministic lock-release delay comfortably below configured lock wait timeout.
- Risk: lock-file lifecycle assertions race with request completion.
  Mitigation: assert behavior via response/result + persisted lineage, not strict timing windows.

## Interfaces / Contracts
- `/v1/responses` request path.
- Session persistence files under `openresponses/sessions/*.jsonl` and lock file `*.lock`.
- Existing weekly-tier selector naming (`tier_weekly_*`).

## ADR
Not required (no new dependencies or architecture/protocol changes).
