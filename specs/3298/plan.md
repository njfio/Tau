# Plan: Issue #3298 - stabilize tau-coding-agent baseline tests

## Approach
1. Reproduce all seven listed failing tests to preserve red evidence.
2. Patch only test fixtures/assertions to match current upstream contracts where behavior changes are intentional.
3. Re-run targeted tests first to confirm each drift area is resolved.
4. Run full `tau-coding-agent` baseline suite to restore mutation precondition.
5. Run fmt/clippy for touched crate.
6. Add focused APO regression tests for historically missed mutation points and run targeted mutation checks.

## Affected Modules
- `crates/tau-coding-agent/src/tests/auth_provider/auth_and_provider/provider_client_and_store.rs`
- `crates/tau-coding-agent/src/tests/auth_provider/commands_and_packages/extensions_and_packages.rs`
- `crates/tau-coding-agent/src/tests/auth_provider/commands_and_packages/gateway_deployment_voice_webhook.rs`
- `crates/tau-coding-agent/src/tests/auth_provider/runtime_and_startup.rs`
- `crates/tau-coding-agent/src/tests/auth_provider/runtime_and_startup/startup_preflight_and_policy.rs`
- `crates/tau-coding-agent/src/live_rl_runtime.rs`
- `crates/tau-coding-agent/tests/cli_integration/tooling_skills/{skills_cli.rs,skills_registry_install.rs,extensions_packages.rs,prompt_audit.rs}`
- `crates/tau-coding-agent/tests/cli_integration/session_runtime.rs`

## Risks & Mitigations
- Risk: masking real regressions by changing tests.
  - Mitigation: verify implementations in source crates (`tau-provider`, `tau-onboarding`, `tau-cli`, `tau-diagnostics`, `tau-tools`) before updating assertions.
- Risk: additional hidden baseline failures after first seven are fixed.
  - Mitigation: run full crate suite and address newly surfaced failures within this issue scope.
- Risk: focused mutation runs are expensive and can stall on long compile/test cycles.
  - Mitigation: run narrowed regex scopes, short explicit test timeouts, and capture outcomes in `/tmp/mutants-3298-*`.

## Contracts / Interfaces
- Credential store keyed payload prefix contract from `tau-provider::credential_store`.
- Startup prompt skill rendering contract from `tau-onboarding::startup_prompt_composition` (summary mode).
- Gateway CLI validation message contract from `tau-cli::validation`.
- Prompt telemetry v1 schema compatibility in `tau-diagnostics::summarize_audit_file`.
- Tool policy schema version from `tau-tools::tool_policy_config::tool_policy_to_json`.
- APO branch behavior in `LiveRlRuntimeBridge::run_live_apo_update` for sample-window and significance thresholds.

## ADR
- Not required: no architecture/dependency/protocol decisions changed.
