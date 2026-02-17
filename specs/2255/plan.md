# Plan #2255

Status: Reviewed
Spec: specs/2255/spec.md

## Approach

1. Add a shared helper in onboarding runtime assembly to derive pre-flight token
   limits from model metadata and existing defaults.
2. Extend `LocalRuntimeAgentSettings` and `build_local_runtime_agent` to carry
   and apply derived pre-flight limits to `AgentConfig`.
3. Wire local runtime startup and training executor startup to use the same
   derivation helper.
4. Add targeted tests:
   - derivation helper behavior
   - local runtime pre-flight rejection behavior
   - training executor path wiring/behavior

## Affected Modules

- `crates/tau-onboarding/src/startup_local_runtime.rs`
- `crates/tau-onboarding/src/startup_local_runtime/tests.rs`
- `crates/tau-coding-agent/src/startup_local_runtime.rs`
- `crates/tau-coding-agent/src/training_runtime.rs`
- `crates/tau-coding-agent/src/tests/auth_provider/runtime_and_startup.rs`

## Risks and Mitigations

- Risk: over-restrictive limits for models with missing catalog metadata.
  - Mitigation: preserve existing defaults when metadata is unavailable.
- Risk: divergence between local runtime and training runtime behavior.
  - Mitigation: centralize derivation logic and reuse in both paths.

## Interfaces / Contracts

- `LocalRuntimeAgentSettings` gains explicit pre-flight limit fields.
- `build_local_runtime_agent` applies pre-flight limits directly to
  `AgentConfig`.
- Derivation helper is deterministic and saturating for edge arithmetic.
