# Plan: Issue #3448 - M298 wave-1 E2E harness and ops dashboard conformance slice

Status: Reviewed

## Approach
1. Baseline contract setup:
   - confirm M298 milestone/index + issue artifact package.
2. E2E harness wave-1:
   - implement minimal `TauE2eHarness` scaffolding in test layer,
   - add deterministic scenarios for gateway lifecycle + agent session flow.
3. Dashboard conformance wave-1:
   - extend ops shell live control/data checks where needed,
   - ensure gateway/dashboard contract markers and mutation flow remain stable.
4. Verification and closeout:
   - execute scoped test matrix first,
   - run required formatting/lint/test gates,
   - record RED/GREEN/REGRESSION evidence in tasks + PR template.

## Affected Modules (expected)
- `tests/e2e/` (new harness/scenarios)
- `tests/integration/` (shared helpers as needed)
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`
- `crates/tau-dashboard-ui/src/tests.rs`
- docs/spec artifacts tied to issue verification

## Risks / Mitigations
- Risk: high compile/runtime cost for new E2E harness paths.
  - Mitigation: keep wave-1 scoped to two scenario groups and use targeted test selectors.
- Risk: flaky behavior from async/live streams.
  - Mitigation: deterministic fixtures, explicit timeouts, no live provider dependencies.
- Risk: over-scoping beyond wave-1.
  - Mitigation: strict in/out boundaries and follow-up issues for remaining scenario groups.

## Interfaces / Contracts
- Test harness contract: deterministic scripted LLM and isolated workspace.
- Dashboard contract: stable ops-shell control/data markers and mutation endpoints.
- Verification contract: AC->conformance->tests traceability in issue artifacts and PR body.
