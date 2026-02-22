# Spec: Issue #3298 - stabilize tau-coding-agent baseline tests to unblock mutation gate

Status: Accepted

## Problem Statement
`cargo mutants` could not execute for #3296 because the unmutated `tau-coding-agent` baseline had seven failing tests. The failures are expectation/data drift versus current runtime/library behavior, which blocks mutation gates and slows delivery cadence.

## Scope
In scope:
- Align `tau-coding-agent` tests with current behavior for:
  - credential store keyed encryption prefix version
  - startup prompt composition skill summary mode
  - gateway OpenResponses auth validation error wording
  - prompt telemetry v1 schema requirements in audit summary fixtures
  - tool policy JSON schema version
- Add APO regression tests to harden mutation-sensitive branches in
  `LiveRlRuntimeBridge::run_live_apo_update`:
  - sample window capping behavior
  - minimum sample and hard-floor thresholds
  - non-significant improvement rejection path
- Re-validate targeted failures and full `tau-coding-agent` baseline.

Out of scope:
- Behavior changes in production logic for provider/gateway/onboarding/diagnostics/tool-policy modules.
- New feature work.

## Acceptance Criteria
### AC-1 credential-store keyed encryption test matches current payload version
Given keyed credential encryption is AES-GCM v2,
when roundtrip unit tests run,
then assertions validate v2 prefix expectations.

### AC-2 startup prompt skill test matches summary-mode composition
Given startup prompt composition renders skills in summary mode,
when activated skill aliases are composed,
then tests assert summary markers instead of full skill body text.

### AC-3 gateway auth validation tests match current CLI contract
Given token/password modes now accept direct value or credential-store id,
when auth inputs are missing,
then tests assert the updated error message contract.

### AC-4 audit summary fixture satisfies prompt telemetry v1 schema
Given prompt telemetry v1 compatibility requires schema version,
when audit summary parses prompt rows,
then test fixtures include schema metadata and prompt counts pass.

### AC-5 tool-policy JSON schema assertion tracks current schema
Given tool policy JSON schema version is now 13,
when tests validate payload shape,
then schema assertion expects 13.

### AC-6 baseline and mutation precondition are restored
Given all expectation drift fixes are applied,
when baseline suite is run,
then targeted failures are green and `cargo test -p tau-coding-agent --bin tau-coding-agent` passes.

### AC-7 APO mutation-sensitive branches have regression coverage
Given historical missed mutants in `run_live_apo_update`,
when regression tests execute,
then capping/threshold/significance branches are exercised and focused mutants are caught or timeout-killed.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
|---|---|---|---|---|---|
| C-01 | AC-1 | Unit/Conformance | keyed encryption mode | roundtrip test runs | encoded payload matches current keyed prefix contract |
| C-02 | AC-2 | Integration/Conformance | activated `checks` skill alias | startup prompt composed | summary markers present and stale full-body assertion removed |
| C-03 | AC-3 | Regression/Conformance | missing token/password under gateway auth modes | validation test runs | error assertion matches current CLI wording |
| C-04 | AC-4 | Functional/Conformance | prompt telemetry v1 rows in audit log | summarize audit file | prompt record counts/provider stats match expected totals |
| C-05 | AC-5 | Unit/Conformance | serialized tool policy JSON | schema assertion runs | schema_version equals 13 |
| C-06 | AC-6 | Regression | full tau-coding-agent test suite | cargo test baseline | suite passes and mutation baseline precondition is satisfied |
| C-07 | AC-7 | Regression/Conformance | > apo_max_samples rollouts | APO update runs | sampled window is capped to configured max |
| C-08 | AC-7 | Regression/Conformance | rollout counts around `apo_min_samples` and hard floor | APO update runs | insufficient-samples reason only when threshold is truly unmet |
| C-09 | AC-7 | Regression/Conformance | candidate improvement with no statistical significance | APO update runs | prompt not adopted and reason reflects no significant improvement |

## Success Metrics / Observable Signals
- `cargo test -p tau-coding-agent --bin tau-coding-agent tests::auth_provider::auth_and_provider::provider_client_and_store::unit_encrypt_and_decrypt_credential_store_secret_roundtrip_keyed`
- `cargo test -p tau-coding-agent --bin tau-coding-agent tests::auth_provider::commands_and_packages::extensions_and_packages::integration_compose_startup_system_prompt_uses_activated_skill_aliases`
- `cargo test -p tau-coding-agent --bin tau-coding-agent tests::auth_provider::commands_and_packages::gateway_deployment_voice_webhook`
- `cargo test -p tau-coding-agent --bin tau-coding-agent tests::auth_provider::runtime_and_startup::functional_summarize_audit_file_aggregates_tool_and_provider_metrics`
- `cargo test -p tau-coding-agent --bin tau-coding-agent tests::auth_provider::runtime_and_startup::startup_preflight_and_policy::unit_tool_policy_to_json_includes_key_limits_and_modes`
- `cargo test -p tau-coding-agent --bin tau-coding-agent`
- `cargo test -p tau-coding-agent --bin tau-coding-agent live_rl_runtime::tests::spec_c09_regression_live_apo_caps_samples_to_max_window`
- `cargo test -p tau-coding-agent --bin tau-coding-agent live_rl_runtime::tests::spec_c10_regression_live_apo_sample_thresholds_respect_min_and_hard_floor`
- `cargo test -p tau-coding-agent --bin tau-coding-agent live_rl_runtime::tests::spec_c11_regression_live_apo_rejects_non_significant_positive_delta`
- `cargo mutants -p tau-coding-agent -f crates/tau-coding-agent/src/live_rl_runtime.rs --re 'live_rl_runtime\\.rs:(803|808|904):.*run_live_apo_update' --baseline skip ...`
