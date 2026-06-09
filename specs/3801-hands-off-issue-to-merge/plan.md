# Plan: Hands-Off Issue-To-Merge Orchestration

## Approach

Layer the new product surface over the existing autonomous coding job runtime.
Do not create a separate agent runner. Add an `AutonomousCodingIssueToMerge`
request/outcome that:

1. Checks for verifier commands and controlled edits.
2. Falls back to existing non-mutating issue intake when authority is missing.
3. Submits a normal autonomous coding job when authority exists.
4. Runs the job immediately through `CodingMissionRunner`.
5. Passes PR publication auth to the PR-ready bundle path for draft PR creation.
6. Optionally calls the existing protected-branch-safe auto-merge request.

## Affected Modules

- `crates/tau-runtime/src/autonomous_coding_jobs_runtime.rs`
- `crates/tau-coding-agent/src/bin/tau_autonomous_coding_job.rs`
- `scripts/dev/test-autonomous-coding-issue-to-merge.sh`
- `docs/guides/autonomous-coding-jobs.md`
- `README.md`
- `specs/3801-hands-off-issue-to-merge/*`

## Risks

- Operators may read "issue-to-merge" as permissionless arbitrary mutation.
  Mitigation: block without verifier/edit authority and state that boundary in
  CLI docs and README.
- Auto-merge may be mistaken for protected-branch bypass. Mitigation: reuse the
  existing `gh pr merge --auto` path and assert no `--admin`.
- Secrets could be persisted if GitHub env is copied into records. Mitigation:
  pass GitHub env only into command execution, never store it in job JSON.

## Verification

Start with focused runtime tests, then CLI binary tests, then a shell integration
that uses a disposable repo and fake `gh` binary to prove draft PR creation,
auto-merge request, and no-authority blocking.
