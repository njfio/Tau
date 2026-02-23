# M291 - Tau E2E PRD Execution

Status: Completed

## Context
`specs/tau-e2e-testing-prd.md` defines end-to-end coverage targets for Tau runtime behavior (gateway lifecycle, sessions, OpenAI compatibility, tools, safety, memory, training, cortex, dashboard, auth, and failure modes). The repository has broad gateway/integration tests, but no single PRD-aligned conformance suite and no complete scenario-to-test traceability.

## Scope
- Execute issue `#3386` as the implementation task under `#3385`/`#3384`.
- Add/extend deterministic E2E tests to cover all PRD scenario groups with explicit IDs.
- Close uncovered scenario gaps or document strict `N/A` justifications when a scenario is not applicable to current runtime contracts.
- Provide CI-tier runnable selectors (`tier_pr`, `tier_nightly`, `tier_weekly`) and verify targeted suites pass.

## Linked Issues
- Epic: #3384
- Story: #3385
- Task: #3386

## Closeout
- Milestone objective delivered via issue `#3386`.
- PRD scenario accounting is tracked in `specs/3386/conformance-matrix.md` with explicit Covered/N/A decisions.
- Tiered runtime suites are implemented as `tier_pr_*`, `tier_nightly_*`, and `tier_weekly_*` tests in `crates/tau-gateway/src/gateway_openresponses/tests.rs`.

## Success Signals
- `specs/3386/spec.md`, `specs/3386/plan.md`, and `specs/3386/tasks.md` exist and are implemented.
- PRD scenario IDs are traceable to executable tests (or justified `N/A`) with no silent gaps.
- `cargo test -p tau-gateway` and targeted `cargo test -p tau-integration-tests` E2E runs pass for new/modified tests.
