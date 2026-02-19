# Plan #2561

## Approach
1. Extend `ProfilePolicyDefaults` with six compaction policy fields and serde defaults.
2. Thread those fields through `LocalRuntimeAgentSettings`.
3. Apply settings in `build_local_runtime_agent` so `AgentConfig` receives policy values.
4. Populate settings from profile defaults in local runtime and from `AgentConfig::default()` in training runtime.
5. Add/update tests for profile parsing/backfill and runtime wiring behavior.

## Affected Modules
- `crates/tau-onboarding/src/startup_config.rs`
- `crates/tau-onboarding/src/startup_local_runtime.rs`
- `crates/tau-onboarding/src/startup_local_runtime/tests.rs`
- `crates/tau-onboarding/src/profile_store.rs`
- `crates/tau-coding-agent/src/startup_local_runtime.rs`
- `crates/tau-coding-agent/src/training_runtime.rs`
- `crates/tau-coding-agent/src/runtime_loop.rs`
- `crates/tau-coding-agent/src/commands.rs`

## Risks & Mitigations
- Risk: profile schema drift breaks legacy profile loading.
  - Mitigation: serde defaults + regression test for legacy profile fixture.
- Risk: runtime behavior changes unexpectedly.
  - Mitigation: preserve existing default values and add conformance tests for explicit override mapping.

## Interfaces / Contracts
- `ProfilePolicyDefaults` serialization contract (backward compatible).
- `LocalRuntimeAgentSettings` wiring contract to `AgentConfig`.
