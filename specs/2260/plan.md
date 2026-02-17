# Plan #2260

Status: Reviewed
Spec: specs/2260/spec.md

## Approach

1. Introduce a testable onboarding command execution path with injected prompt
   callback and output sink.
2. Keep existing `execute_onboarding_command` as thin wrapper over real stdin/stdout
   adapters.
3. Wire wizard-selected workspace into onboarding root resolution for report and
   bootstrap persistence paths.
4. Add deterministic command-level tests first (RED), then implement minimal production
   changes (GREEN), then run crate regressions.

## Affected Modules

- `crates/tau-onboarding/src/onboarding_command.rs`
- `crates/tau-onboarding/src/onboarding_report.rs` (if report-path helper needs
  workspace-root variant)
- `specs/2260/*`

## Risks and Mitigations

- Risk: interactive refactor could change current prompt text/ordering.
  - Mitigation: keep prompt strings stable and cover with deterministic tests.
- Risk: workspace-root changes could break non-interactive defaults.
  - Mitigation: explicit regression test for non-interactive report path behavior.

## Interfaces / Contracts

- Public `execute_onboarding_command(&Cli) -> Result<()>` remains unchanged.
- New internal test seam for prompt/output injection is allowed for deterministic
  tests.
- Onboarding persistence root for interactive flow must follow wizard-selected
  workspace when provided.
