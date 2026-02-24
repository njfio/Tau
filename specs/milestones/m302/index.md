# M302 - Ops dashboard workflow depth (control-action outcomes + fail-closed forms)

Status: Active

## Context
M302 deepens the `/ops` control-action workflow by making outcome state explicit
for operators and fail-closed for malformed form submissions. The slice is
focused on deterministic redirect contracts and UI-visible status markers for
control actions.

Primary sources:
- `crates/tau-gateway/src/gateway_openresponses/ops_dashboard_shell.rs`
- `crates/tau-gateway/src/gateway_openresponses/ops_shell_controls.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`
- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-dashboard-ui/src/tests.rs`

## Issue Hierarchy
- Epic: #3464
- Story: #3465
- Task: #3466

## Scope
- Add deterministic control-action outcome query contract to `/ops` redirects.
- Fail closed on missing/invalid control-action form submissions using
  redirect+marker semantics instead of raw error responses.
- Render control-action outcome markers in the ops shell command-center panel.
- Add conformance coverage for missing, invalid, and applied action paths.

## Exit Criteria
- `specs/3466/spec.md` is implemented with AC-to-test evidence.
- `POST /ops/control-action` returns deterministic `303` redirects for missing,
  invalid, and applied form outcomes.
- Ops shell renders normalized action-outcome markers for operators.
- `tau-gateway` and `tau-dashboard-ui` conformance/regression selectors pass.
