# Plan 3811: Actionable Verifier Missing Inputs

## Approach

Keep the existing `AutonomousCodingVerifierPlan` schema and make the
repo-aware verifier derivation helper return diagnostics alongside concrete
commands.

1. Add a small derivation outcome that contains `commands` and
   `missing_inputs`.
2. When Cargo metadata has no matching package token, report the missing real
   package input and include available package context.
3. When no exact safe quoted/backticked test filter is present, report the
   missing filter contract.
4. Preserve `derive_repo_aware_code_verifier_commands` as the command-only
   wrapper used by the existing job path.
5. Merge derivation diagnostics into the verifier plan without adding new
   persisted fields.

## Affected Files

- `crates/tau-runtime/src/autonomous_coding_jobs_runtime.rs`
- `crates/tau-runtime/src/autonomous_coding_jobs_runtime/verifier_derivation.rs`
- `crates/tau-runtime/src/autonomous_coding_jobs_runtime/tests/repo_aware_verifier.rs`
- `docs/guides/autonomous-coding-jobs.md`
- `specs/3811-actionable-verifier-missing-inputs/*`

## Risks

- Risk: missing-input strings become too noisy in large workspaces.
  Mitigation: include only a short package sample.
- Risk: diagnostics accidentally imply mutation authority.
  Mitigation: command derivation remains fail-closed and blocked outcomes still
  do not submit jobs.

## Interfaces

No CLI or JSON schema changes. This only improves `missing_inputs` and
`next_action` content inside the existing verifier plan.
