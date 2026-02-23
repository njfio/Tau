# M291 - Tau E2E PRD Execution

Status: In Progress

## Context
`specs/tau-e2e-testing-prd.md` defines end-to-end coverage targets for Tau runtime behavior (gateway lifecycle, sessions, OpenAI compatibility, tools, safety, memory, training, cortex, dashboard, auth, and failure modes). The repository has broad gateway/integration tests, but no single PRD-aligned conformance suite and no complete scenario-to-test traceability.

## Scope
- Execute issue `#3386` as the initial implementation task under `#3385`/`#3384`.
- Execute issue `#3388` to close remaining P0 OpenAI compatibility scenario gaps (`O3-06`, `O3-07`, `O3-08`, `O3-10`).
- Execute issue `#3390` to close Scenario Group 6 tool-call navigation gaps (`B6-03`, `B6-04`, `B6-05`).
- Execute issue `#3392` to close Scenario Group 15 lock-contention chaos gap (`CH15-05`).
- Execute issue `#3394` to close Scenario Group 15 memory-pressure chaos gap (`CH15-06`).
- Execute issue `#3396` to close Scenario Group 14 MCP inventory gap (`E14-04`).
- Add/extend deterministic E2E tests to cover all PRD scenario groups with explicit IDs.
- Close uncovered scenario gaps or document strict `N/A` justifications when a scenario is not applicable to current runtime contracts.
- Provide CI-tier runnable selectors (`tier_pr`, `tier_nightly`, `tier_weekly`) and verify targeted suites pass.

## Linked Issues
- Epic: #3384
- Story: #3385
- Task: #3386
- Task: #3388
- Task: #3390
- Task: #3392
- Task: #3394
- Task: #3396

## Closeout
- Phase 1 delivered via issue `#3386` and PR `#3387`.
- Phase 2 delivered via issue `#3388`, closing unresolved P0 OpenAI compatibility scenarios (`O3-06`, `O3-07`, `O3-08`, `O3-10`).
- Phase 3 delivered via issue `#3390`, closing unresolved tool-call-driven branch/navigation scenarios (`B6-03`, `B6-04`, `B6-05`).
- Phase 4 delivered via issue `#3392`, closing lock-contention chaos scenario coverage (`CH15-05`).
- Phase 5 delivered via issue `#3394`, closing memory-pressure chaos scenario coverage (`CH15-06`).
- Phase 6 in progress via issue `#3396` for MCP inventory scenario coverage (`E14-04`).

## Success Signals
- `specs/3386/spec.md`, `specs/3386/plan.md`, and `specs/3386/tasks.md` exist and are implemented.
- PRD scenario IDs are traceable to executable tests (or justified `N/A`) with no silent gaps.
- `cargo test -p tau-gateway` and targeted `cargo test -p tau-integration-tests` E2E runs pass for new/modified tests.
