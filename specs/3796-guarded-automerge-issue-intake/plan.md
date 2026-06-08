# Plan: Guarded Auto-Merge And Issue Intake

GitHub Issue: #3796
Status: Implemented

## Approach

Extend the autonomous coding job runtime rather than adding a separate merger:

- Add `request_auto_merge` to the runtime with explicit policy/auth/PR URL
  gates.
- Store merge evidence under the existing autonomous job record/status JSON.
- Execute `gh pr merge <pr-url> --auto --<method>` only after all gates pass.
- Keep the command free of admin/protection bypass flags.

Add arbitrary issue intake as a durable non-mutating path:

- Persist issue context and authority requirements under
  `<state-dir>/issue-intake/<intake-id>.json`.
- Emit status JSON that says the intake is blocked until verifier and edit
  authority are provided.
- Do not create a coding mission or write to the repo in this path.

Extend `tau_autonomous_coding_job` with:

- `auto-merge`
- `intake-issue`
- `intake-status`

## Affected Modules

- `crates/tau-runtime/src/autonomous_coding_jobs_runtime.rs`
- `crates/tau-coding-agent/src/bin/tau_autonomous_coding_job.rs`
- `scripts/dev/test-autonomous-coding-automerge-intake.sh`
- `docs/guides/autonomous-coding-jobs.md`
- `specs/3796-guarded-automerge-issue-intake/*`

## Risks And Mitigations

- Risk: auto-merge is mistaken for branch-protection bypass. Mitigation:
  require `--auto`, reject missing auth/policy, never emit `--admin`, and
  document the boundary.
- Risk: arbitrary issue intake is overstated as code-solving. Mitigation:
  persist a blocked authority plan unless verifier/edit authority is present.
- Risk: shell integration depends on live GitHub. Mitigation: use a fake `gh`
  executable that captures argv for deterministic proof.

## Interfaces

- `request_auto_merge(request) -> AutonomousCodingAutoMergeOutcome`
- `intake_issue_without_authority(request) -> AutonomousCodingIssueIntakeOutcome`
- CLI JSON output remains machine-readable and includes status reason codes.

## ADR

No ADR required: this uses existing GitHub CLI behavior and adds no new
dependency or protocol.
