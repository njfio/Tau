# Plan: Issue #3390

## Approach
1. Add RED assertions for `B6-03/04/05` in a nightly-tier gateway E2E test using scripted tool-call turns.
2. Extend fixture pipeline tools with deterministic `branch`, `undo`, and `redo` behaviors backed by `tau-session` store/navigation helpers.
3. Validate resulting branch session and navigation transitions through gateway/runtime artifacts.
4. Update conformance mapping for B6 rows and re-run verification gates.

## Affected Modules
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`
- `specs/3386/conformance-matrix.md`
- `specs/3390/spec.md`
- `specs/3390/plan.md`
- `specs/3390/tasks.md`
- `specs/3390/conformance-matrix.md`
- `specs/milestones/m291/index.md`

## Risks and Mitigations
- Risk: fixture implementation drifts from runtime session semantics.
  Mitigation: use exported `tau-session` APIs (`SessionStore`, navigation helpers) instead of ad-hoc state simulation.
- Risk: additional tool fixtures make nightly tests flaky.
  Mitigation: keep scripted responses deterministic and avoid wall-clock timing dependencies.

## Interfaces / Contracts
- Gateway OpenResponses request loop (`/v1/responses`) with tool-call turns.
- Gateway session artifacts (`openresponses/sessions/*.jsonl`) and session navigation state.
- Existing tier naming contract (`tier_nightly_*`).

## ADR
Not required (no new dependency and no architectural protocol changes).
