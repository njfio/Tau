# Spec: Issue #3386 - Implement Tau E2E PRD scenario coverage suite

Status: Implemented

## Problem Statement
`specs/tau-e2e-testing-prd.md` defines comprehensive end-to-end scenario coverage for Tau runtime behavior, but current tests are distributed across modules and do not provide complete PRD-traceable execution for every scenario group with deterministic tiered execution.

## Scope
In scope:
- Add deterministic E2E tests and helpers that execute PRD scenario groups against real gateway/runtime paths with mocked/scripted LLM behavior.
- Ensure each PRD scenario ID has a mapped executable test or explicit justified `N/A`.
- Close identified gaps by adding missing tests and fixing runtime/test defects exposed by those tests.
- Add tier selectors (`tier_pr`, `tier_nightly`, `tier_weekly`) to support CI partitioning.

Out of scope:
- External live-provider API calls (OpenAI/Anthropic/Google network integration).
- Non-deterministic load benchmarking beyond chaos/reliability conformance checks.

## Acceptance Criteria
### AC-1 PRD scenario traceability is complete
Given `specs/tau-e2e-testing-prd.md`,
when conformance artifacts for `#3386` are reviewed,
then every scenario ID across Groups 1-15 is mapped to at least one executable test or a justified `N/A` entry with rationale.

### AC-2 P0 scenarios are executable and green in deterministic runs
Given the new/updated E2E tests,
when PR-tier commands are run,
then P0 scenarios (gateway lifecycle, session flow, OpenAI compatibility, tool pipeline, safety) pass with deterministic scripted providers.

### AC-3 P1/P2/P3 scenario gaps are closed or explicitly justified
Given nightly/weekly scenario groups,
when targeted suites are run,
then each uncovered scenario is either implemented as passing test coverage or documented as `N/A` with concrete runtime-contract justification.

### AC-4 Test tier evidence and commands are captured
Given AGENTS testing contract,
when verification is performed,
then RED/GREEN/REGRESSION evidence and tier outcomes are captured for the implemented suite with passing command output summaries.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
|---|---|---|---|---|---|
| C-01 | AC-1 | Conformance | PRD scenario tables | build mapping matrix | every scenario ID is mapped or justified `N/A` |
| C-02 | AC-2 | Functional/E2E | scripted LLM + isolated state | run P0 scenario tests | all P0 tests pass deterministically |
| C-03 | AC-2 | Regression | malformed/unauthorized inputs | run P0 negative-path tests | fail-closed behavior remains stable |
| C-04 | AC-3 | Functional/E2E | P1/P2/P3 scenario set | run nightly/weekly selectors | missing scenario gaps are closed or justified |
| C-05 | AC-4 | Process | test commands and logs | execute verify commands | RED/GREEN/REGRESSION evidence exists |

## Success Metrics / Observable Signals
- `specs/3386/spec.md`, `specs/3386/plan.md`, `specs/3386/tasks.md` present and status advanced to Implemented at close.
- PRD scenario coverage matrix exists with 100% scenario accounting (`specs/3386/conformance-matrix.md`).
- Targeted new/modified scenario tests pass in `cargo test`.
