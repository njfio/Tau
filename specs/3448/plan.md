# Plan: Issue #3448 - M298 wave-1 E2E harness and ops dashboard conformance slice

Status: Implemented

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

## Execution Summary
1. Added a scoped in-file `TauE2eHarness` in `crates/tau-gateway/src/gateway_openresponses/tests.rs` for deterministic gateway lifecycle/session-flow execution with token auth and isolated temp workspace.
2. Added wave-1 conformance tests:
   - `integration_spec_3448_c02_tau_e2e_harness_runs_gateway_lifecycle_and_session_flow`
   - `integration_spec_3448_c03_tau_e2e_harness_keeps_ops_control_and_dashboard_live_contracts`
3. Preserved ops-dashboard live control/read-model coverage by validating shell markers, control-action redirect behavior, and dashboard health payload contracts.
4. Verified formatting and targeted regression tests; mapped ACs to concrete test evidence in `specs/3448/spec.md` and `specs/3448/tasks.md`.

## Verification Notes
- `cargo fmt --check` passed after formatting (`cargo fmt --all`).
- Targeted test and regression commands passed under `CARGO_TARGET_DIR=target-fast` (see tasks evidence matrix).
- `CARGO_TARGET_DIR=target-fast cargo clippy -p tau-gateway --tests --no-deps -- -D warnings` passed (isolated target cache avoids local `target/debug` stall behavior).
