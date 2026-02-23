# Plan: Issue #3394

## Approach
1. Extend `tier_weekly_ch15_chaos_matrix` with a deterministic `CH15-06` block that creates `100` unique sessions with multi-turn history via `/v1/responses`.
2. Assert high-volume request success, validate resulting session discovery/persistence, and confirm post-pressure responsiveness via follow-up request.
3. Update `CH15-06` conformance mapping from `N/A` to `Covered`.
4. Run verification gates for touched scope.

## Affected Modules
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`
- `specs/3386/conformance-matrix.md`
- `specs/3394/spec.md`
- `specs/3394/plan.md`
- `specs/3394/tasks.md`
- `specs/3394/conformance-matrix.md`
- `specs/milestones/m291/index.md`

## Risks and Mitigations
- Risk: pressure path introduces long-running/flaky weekly tests.
  Mitigation: keep deterministic mocked provider and bounded request count (`100` sessions, fixed turns).
- Risk: pressure request volume trips rate-limit protections.
  Mitigation: configure test runtime with explicit high request budget for this scenario.

## Interfaces / Contracts
- `/v1/responses` request path.
- `/gateway/sessions` list endpoint.
- Session persistence files under `openresponses/sessions/*.jsonl`.
- Existing weekly-tier selector naming (`tier_weekly_*`).

## ADR
Not required (no dependency/protocol/architecture change).
