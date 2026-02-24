# Tasks: Issue #3444 - M293 operator integration closure set

Status: Implemented

## Ordered Tasks
1. [x] T1 (RED, Conformance): capture failing docs capability/archive contract output for README markers.
2. [x] T2 (GREEN, Docs): update README with exact required operator-surface and canonical-link markers.
3. [x] T3 (RED, Unit/Functional): add failing tests for new TUI live shell mode parsing/render behavior.
4. [x] T4 (GREEN, Implementation): implement TUI live shell mode and CLI wiring.
5. [x] T5 (RED, Integration): add failing gateway test for ops control-action form submission flow.
6. [x] T6 (GREEN, Implementation): implement ops control-action handler/route and shell form wiring.
7. [x] T7 (RED, Regression): add failing verification assertions for RL hardening + live-auth readiness steps.
8. [x] T8 (GREEN, Scripts/Docs): implement readiness script/runbook updates with pass/skip evidence semantics.
9. [x] T9 (VERIFY): run targeted test/script matrix and record RED/GREEN/REGRESSION evidence.
10. [x] T10 (VERIFY): set spec status to `Implemented` once all ACs are met.

## Verification Evidence (2026-02-24)
### RED
- `bash scripts/dev/test-docs-capability-archive.sh`
- Result (pre-fix): failed due to missing required README operator-surface markers.
- Added tests first for new behaviors:
  - `spec_c03_parse_args_accepts_shell_live_mode_and_state_dir`
  - `regression_parse_args_rejects_shell_live_state_dir_without_value`
  - `functional_live_shell_frame_loads_dashboard_and_training_artifacts`
  - `regression_live_shell_frame_handles_missing_artifacts_without_panicking`
  - `integration_ops_shell_control_action_form_submits_dashboard_mutation_and_redirects`
  - `functional_spec_2810_c01_c02_c03_ops_shell_control_markers_reflect_dashboard_control_snapshot`
  - skip-path coverage in `scripts/verify/test-m296-ga-readiness-gate.sh`

### GREEN
- `bash scripts/dev/test-docs-capability-archive.sh`
- `cargo fmt --all`
- `cargo fmt --check`
- `cargo clippy -p tau-tui -p tau-dashboard-ui -p tau-gateway -- -D warnings`
- `cargo test -p tau-tui`
- `cargo test -p tau-dashboard-ui`
- `cargo test -p tau-gateway integration_ops_shell_control_action_form_submits_dashboard_mutation_and_redirects`
- `cargo test -p tau-gateway functional_spec_2810_c01_c02_c03_ops_shell_control_markers_reflect_dashboard_control_snapshot`
- `bash scripts/verify/test-m296-ga-readiness-gate.sh`
- `bash scripts/verify/m296-ga-readiness-gate.sh`
- Result: all commands above passed; `auth_live_validation_matrix` reported deterministic `skip` in GA gate with passing signoff criteria.

### REGRESSION
- `jq '{overall, steps: [.steps[] | {id,status}], signoff_criteria: [.signoff_criteria[] | {id,status}]}' artifacts/operator-ga-readiness/verification-report.json`
- `jq '{overall, steps: [.steps[] | {id,status}]}' artifacts/operator-maturity-wave/verification-report.json`
- Result:
  - `artifacts/operator-ga-readiness/verification-report.json`: `overall=pass`, signoff criteria all `pass`, `auth_live_validation_matrix=skip` (accepted by criterion).
  - `artifacts/operator-maturity-wave/verification-report.json`: `overall=pass`, all six maturity steps `pass`.

## AC Coverage Map
| AC | Case(s) | Evidence |
| --- | --- | --- |
| AC-1 | C-01 | `bash scripts/dev/test-docs-capability-archive.sh` |
| AC-2 | C-02 | `cargo test -p tau-tui` |
| AC-3 | C-03 | `cargo test -p tau-gateway integration_ops_shell_control_action_form_submits_dashboard_mutation_and_redirects` |
| AC-4 | C-04 | `bash scripts/verify/m296-ga-readiness-gate.sh` + GA report `rl_hardening_live_benchmark_proof=pass` |
| AC-5 | C-05 | `bash scripts/verify/m296-ga-readiness-gate.sh` + GA report `auth_live_validation_matrix=skip` + signoff `auth_live_validation=pass` |

## Test Tier Matrix
| Tier | ✅/❌/N/A | Tests / Evidence | N/A Why |
|---|---|---|---|
| Unit | ✅ | `cargo test -p tau-tui` (arg parser + live frame unit coverage) | |
| Property | N/A | No new randomized invariant/parsing surface introduced | Not required for this delta |
| Contract/DbC | N/A | No new contracts crate annotations added | Existing API contracts unchanged |
| Snapshot | N/A | No snapshot harness introduced for this issue | Structured markers covered by functional/integration tests |
| Functional | ✅ | `cargo test -p tau-tui`, `cargo test -p tau-dashboard-ui` | |
| Conformance | ✅ | docs contract script + AC coverage map commands above | |
| Integration | ✅ | gateway `/ops/control-action` integration test | |
| Fuzz | N/A | No new untrusted parser/input engine added | Existing fuzz scope unchanged |
| Mutation | N/A | Not run for this issue scope; critical-path gate already represented in M295/M296 reports | Follow-up can run `cargo mutants --in-diff` if required by PR gate |
| Regression | ✅ | `bash scripts/verify/test-m296-ga-readiness-gate.sh` + GA report checks | |
| Performance | N/A | No hotspot/perf-sensitive algorithm changes | No new perf baseline required |
