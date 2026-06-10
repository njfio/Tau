# Plan 3807: Intake Rerun Command

## Approach

Use persisted intake JSON as the operator handoff. Add only the missing
ingredient needed for an exact rerun command: the original issue body.

1. Add backward-compatible `issue_body` to
   `AutonomousCodingIssueIntakeOutcome`.
2. Populate it from `AutonomousCodingIssueIntakeRequest`.
3. Extend `tau-unified intake` to compute, not persist, a rerun command.
4. Gate command generation to `decision=needs_authority`, non-empty issue body,
   and concrete verifier commands that do not contain placeholder brackets.
5. Shell-quote every argv part with Python `shlex.join`.

## Affected Files

- `crates/tau-runtime/src/autonomous_coding_jobs_runtime.rs`
- `scripts/run/tau-unified.sh`
- `scripts/run/test-tau-unified.sh`
- `docs/guides/autonomous-coding-jobs.md`
- `specs/3807-intake-rerun-command/*`

## Risks

- Risk: command output could encourage mutation from underspecified intake.
  Mitigation: emit only for `needs_authority` records with concrete verifier
  commands and original body.
- Risk: shell quoting mistakes in issue text.
  Mitigation: build argv and use `shlex.join` in the Python renderer.
- Risk: schema drift for older intake JSON.
  Mitigation: `issue_body` uses a serde default and legacy test coverage.

## Interfaces

No new CLI command. `tau-unified intake <id>` gains one line:

`tau-unified: autonomous_coding.intake.rerun_command=<command|none>`
