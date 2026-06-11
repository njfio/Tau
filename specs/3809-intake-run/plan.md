# Plan 3809: Intake Run

## Approach

Keep `issue-to-merge` as the single mutation path. `intake-run` only loads
persisted intake, validates that the record is eligible, fills the existing
`AutonomousCodingIssueToMergeRequest`, and delegates.

1. Add `IssueIntakeRunArgs` to `tau_autonomous_coding_job`.
2. Parse controlled edits and provider repair authority exactly like
   `issue-to-merge`.
3. Validate persisted intake:
   - decision must be `needs_authority` or `ready_to_run`;
   - issue body, issue URL/title, repo path, and intake id must be present;
   - verifier commands must come from explicit flags or concrete persisted
     suggestions;
   - edit/provider authority must be present before mutation.
4. Add `tau-unified intake-run` wrapper with state-dir defaults and pass-through
   authority flags.
5. Test helper logic in the binary and wrapper delegation in the shell contract.

## Affected Files

- `crates/tau-coding-agent/src/bin/tau_autonomous_coding_job.rs`
- `scripts/run/tau-unified.sh`
- `scripts/run/test-tau-unified.sh`
- `docs/guides/autonomous-coding-jobs.md`
- `specs/3809-intake-run/*`

## Risks

- Risk: command accidentally mutates from vague intake.
  Mitigation: explicit decision and concrete-verifier gates before building the
  run request.
- Risk: duplicate logic with `issue-to-merge`.
  Mitigation: `intake-run` builds the same request type and delegates to the
  same runtime method.
- Risk: operator-supplied verifier is unsafe.
  Mitigation: it is explicit operator authority; placeholder persisted
  verifiers remain rejected.

## Interfaces

- `tau-autonomous-coding-job intake-run --intake-id <id> [authority flags]`
- `scripts/run/tau-unified.sh intake-run <id> [authority flags]`
