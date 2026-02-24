# Spec: Issue #3444 - M293 operator integration closure set

Status: Implemented

## Problem Statement
Current operator delivery still has concrete integration gaps: the README fails docs-capability contract checks, `tau-tui` shell mode is fixture-only, dashboard control actions in `/ops` are marker-only and not wired as live mutations in the shell workflow, and verification gates do not yet include explicit RL hardening/live-auth validation expansion checks.

## Scope
In scope:
- Fix README markers/links so docs capability/archive contract tests pass.
- Add a state-backed live TUI shell path that renders operator shell data from real runtime artifacts.
- Add live dashboard control-action submission flow in `/ops` shell path.
- Expand RL hardening and live-auth validation checks in readiness verification flow and docs.
- Add or update tests for each behavior change.

Out of scope:
- Replacing the entire web dashboard with a SPA frontend runtime.
- Introducing new external dependencies.
- Live third-party provider network calls in CI-required paths.

## Acceptance Criteria
### AC-1 README contract markers pass docs capability/archive checks
Given repository docs contract scripts,
when `scripts/dev/test-docs-capability-archive.sh` is run,
then it exits zero with README sections/markers present.

### AC-2 TUI has a real state-backed shell mode
Given runtime dashboard artifacts under `.tau/dashboard`,
when `tau-tui` live shell mode is executed,
then the rendered shell surfaces status/auth/training/alerts/actions from live artifact data rather than only deterministic fixtures.

### AC-3 Dashboard ops control actions are wired to live mutation flow
Given `/ops` command center control actions,
when an operator submits pause/resume/refresh via shell controls,
then gateway action runtime is invoked and control-state mutation is persisted with redirect back to ops shell.

### AC-4 RL hardening verification expands readiness evidence
Given M296 readiness execution,
when readiness scripts run,
then RL hardening checks beyond baseline harness execution are included and captured in artifacts.

### AC-5 Live auth validation matrix checks are integrated into readiness flow
Given readiness execution in environments with configured live auth validation inputs,
when readiness scripts run,
then live auth validation checks execute with deterministic pass/skip semantics and are reported.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Conformance | README + docs contract script | run docs capability/archive test | script passes |
| C-02 | AC-2 | Functional | dashboard state artifacts exist | run `tau-tui` live shell mode | output reflects artifact state values |
| C-03 | AC-3 | Integration | ops shell controls and gateway state dir | submit control action form | control state/audit artifacts update |
| C-04 | AC-4 | Regression | M296 gate script | run gate script | RL hardening step appears and passes/skips with evidence |
| C-05 | AC-5 | Regression | M296 gate script + auth env toggles | run gate script | live auth step appears and reports pass/skip semantics |

## Success Metrics / Observable Signals
- `scripts/dev/test-docs-capability-archive.sh` passes.
- `cargo test -p tau-tui` and touched gateway tests pass.
- `scripts/verify/m296-ga-readiness-gate.sh` passes with added RL/auth expansion step logs.
- Runbooks and README reflect the new integrated operator flow.

## Implementation Verification (2026-02-24)
| AC | Result | Verification |
| --- | --- | --- |
| AC-1 | ✅ | `bash scripts/dev/test-docs-capability-archive.sh` |
| AC-2 | ✅ | `cargo test -p tau-tui` (includes `functional_live_shell_frame_loads_dashboard_and_training_artifacts`) |
| AC-3 | ✅ | `cargo test -p tau-gateway integration_ops_shell_control_action_form_submits_dashboard_mutation_and_redirects` and `cargo test -p tau-gateway functional_spec_2810_c01_c02_c03_ops_shell_control_markers_reflect_dashboard_control_snapshot` |
| AC-4 | ✅ | `bash scripts/verify/m296-ga-readiness-gate.sh` + `artifacts/operator-ga-readiness/verification-report.json` (`rl_hardening_live_benchmark_proof=pass`) |
| AC-5 | ✅ | `bash scripts/verify/m296-ga-readiness-gate.sh` + `artifacts/operator-ga-readiness/verification-report.json` (`auth_live_validation_matrix=skip`, criterion `auth_live_validation=pass`) |
