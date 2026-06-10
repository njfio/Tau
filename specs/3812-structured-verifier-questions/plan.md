# Plan 3812: Structured Repo-Aware Verifier Questions

## Approach

Use the existing `clarifying_questions` array. Do not add schema fields.

1. Promote the repo-aware missing-input contract strings to reusable constants.
2. Pass the computed verifier plan into `issue_intake_clarifying_questions`.
3. When verifier authority is missing and the plan reports repo-aware package or
   filter inputs, append stable reason-code questions:
   `repo_aware_cargo_package` and `repo_aware_test_filter`.
4. Keep generic `verifier_command` and `mutation_authority` questions for
   backward-compatible operator flows.

## Affected Files

- `crates/tau-runtime/src/autonomous_coding_jobs_runtime.rs`
- `crates/tau-runtime/src/autonomous_coding_jobs_runtime/verifier_derivation.rs`
- `crates/tau-runtime/src/autonomous_coding_jobs_runtime/tests/repo_aware_verifier.rs`
- `docs/guides/autonomous-coding-jobs.md`
- `specs/3812-structured-verifier-questions/*`

## Risks

- Risk: extra questions duplicate generic verifier prompts.
  Mitigation: keep generic prompts for compatibility and add stable
  repo-specific reason codes for automation.
- Risk: string matching drifts from derivation.
  Mitigation: reuse constants from the verifier derivation module.

## Interfaces

No new CLI flags or persisted fields. Existing `clarifying_questions` records
carry additional reason codes.
