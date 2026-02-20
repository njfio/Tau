# Plan: Issue #2810 - Command-center control-mode/action SSR markers

## Approach
1. Add RED integration tests for `/ops` shell expecting control-mode/action/last-action marker contracts.
2. Extend `tau-dashboard-ui` shell context with command-center control payload and deterministic marker rendering.
3. Map gateway dashboard snapshot control + health reports into UI control marker payload.
4. Re-run phase-1A..1G regression suites.
5. Run fmt/clippy/tests/mutation gates and set spec status to `Implemented`.

## Affected Modules
- `specs/milestones/m136/index.md`
- `specs/2810/spec.md`
- `specs/2810/plan.md`
- `specs/2810/tasks.md`
- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-gateway/src/gateway_openresponses.rs`
- `crates/tau-gateway/src/gateway_openresponses/dashboard_status.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`

## Risks and Mitigations
- Risk: large `gateway_openresponses.rs` growth regresses oversized-file guard.
  - Mitigation: keep logic in `dashboard_status.rs` helper and call from existing renderer.
- Risk: marker additions destabilize existing shell contract tests.
  - Mitigation: preserve existing IDs/attributes and run targeted/full regression suites.

## Interface and Contract Notes
- Extends command-center snapshot payload in `tau-dashboard-ui` with control/action marker fields.
- No endpoint/protocol/schema changes.
