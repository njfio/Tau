# Spec: Issue #3420 - Real TUI shell, RL end-to-end harness, and auth workflow conformance

Status: Implemented

## Problem Statement
Operators requested concrete maturity upgrades instead of component-only surfaces:
- `tau-tui` must be a practical operator shell rather than only a frame demo,
- RL capability must be runnable end-to-end (not only primitive modules),
- auth workflows must have explicit comprehensive validation coverage.

## Scope
In scope:
- Add a real operator-shell mode to `tau-tui` with structured, connected panels and deterministic rendering contracts.
- Add an executable RL end-to-end harness that ties training runner/store + algorithm summaries into a verifiable artifact.
- Add/extend auth conformance tests to validate provider auth mode matrix and gateway auth/session workflows.
- Add/update docs/spec artifacts required for process traceability.

Out of scope:
- External provider live-network integration tests that require production credentials.
- Full dashboard product redesign outside TUI and auth/rl scope.
- New cryptographic/auth protocol changes.

## Acceptance Criteria
### AC-1 `tau-tui` provides a practical operator-shell mode
Given a local operator starts `tau-tui`,  
when shell mode is selected,  
then output includes structured runtime panels (status/auth/training/alerts/actions) with deterministic rendering and tests.

### AC-2 RL end-to-end harness is executable and deterministic
Given training crates and deterministic fixture inputs,  
when the RL harness runs,  
then it executes rollout/training flow, computes RL summaries, and writes a deterministic artifact with pass/fail indicators.

### AC-3 Auth workflows are comprehensively validated
Given supported auth modes and gateway auth/session contracts,  
when conformance tests run,  
then provider mode matrix + gateway auth/session lifecycle behaviors are covered with deterministic pass/fail evidence.

### AC-4 Verification evidence is runnable from repository root
Given this issue change set,  
when scoped test commands are executed,  
then all new/changed tests pass and map to C-01..n cases.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
|---|---|---|---|---|---|
| C-01 | AC-1 | Functional/Conformance | `tau-tui` shell mode args | run shell render | includes expected panel headers and stable layout lines |
| C-02 | AC-1 | Regression | invalid shell args | run CLI parse | fails with explicit usage/error |
| C-03 | AC-2 | Integration/Conformance | deterministic RL fixture dataset | run RL e2e harness | emits artifact with rollout summary + GAE/PPO summary |
| C-04 | AC-2 | Regression | malformed RL config/path | run harness | fails closed with explicit reason code/message |
| C-05 | AC-3 | Functional/Conformance | provider auth mode matrix inputs | run auth conformance suite | all supported mode combinations validated and reported |
| C-06 | AC-3 | Integration/Conformance | gateway auth/session flow | execute deterministic gateway auth/session test path | session issuance/authorized/unauthorized cases pass |
| C-07 | AC-4 | Verify | scoped crate commands | run verification commands | all new tests pass and map to ACs |

## Success Metrics / Observable Signals
- `tau-tui` has a documented/runnable shell mode beyond static demo behavior.
- RL e2e harness produces deterministic artifact output for local verification.
- Auth conformance suite covers provider matrix and gateway auth/session lifecycle.
- PR includes RED/GREEN evidence and AC-to-test mapping.

## Implementation Evidence
### C-01/C-02 (`tau-tui` shell mode + parse regression)
- `cargo test -p tau-tui --tests -- --nocapture`
- Key tests:
  - `crates/tau-tui/src/main.rs`: `spec_c01_parse_args_accepts_shell_mode_and_overrides`
  - `crates/tau-tui/src/main.rs`: `spec_c01_shell_renderer_includes_all_operator_panels`
  - `crates/tau-tui/src/main.rs`: `spec_c02_parse_args_rejects_shell_profile_without_value`
  - `crates/tau-tui/tests/tui_demo_smoke.rs`: `conformance_tui_shell_binary_renders_operator_panels`

### C-03/C-04 (RL deterministic e2e harness + fail-closed path)
- `cargo test -p tau-trainer --test rl_e2e_harness -- --nocapture`
- `cargo run -p tau-trainer --bin rl_e2e_harness -- --run-id m295-verification --output-dir artifacts/rl-e2e`
- Key tests:
  - `crates/tau-trainer/tests/rl_e2e_harness.rs`: `spec_c03_rl_e2e_harness_emits_deterministic_artifact_with_rollout_gae_and_ppo_summaries`
  - `crates/tau-trainer/tests/rl_e2e_harness.rs`: `regression_spec_c04_rl_e2e_harness_rejects_output_path_that_is_a_file`

### C-05/C-06 (auth workflow conformance)
- `cargo test -p tau-provider --test auth_workflow_conformance -- --nocapture`
- `cargo test -p tau-gateway conformance_gateway_auth_session_endpoint_enforces_mode_and_password_lifecycle_contracts -- --nocapture`
- Key tests:
  - `crates/tau-provider/tests/auth_workflow_conformance.rs`: `spec_c05_provider_auth_matrix_contract_is_complete_and_deterministic`
  - `crates/tau-provider/tests/auth_workflow_conformance.rs`: `conformance_provider_auth_matrix_matches_expected_support_by_provider`
  - `crates/tau-gateway/src/gateway_openresponses/tests.rs`: `conformance_gateway_auth_session_endpoint_enforces_mode_and_password_lifecycle_contracts`

### C-07 (root verification)
- `./scripts/verify/m295-operator-maturity-wave.sh`
- Artifact: `artifacts/operator-maturity-wave/verification-report.json`
