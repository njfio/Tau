# Plan: Issue #3802 - Provider Repair Inside Durable Autonomous Coding Jobs

## Approach

Keep the product loop on `AutonomousCodingJobRuntime` and `CodingMissionRunner`.
Add a provider repair adapter contract instead of creating a second agent
runner. The runtime writes a JSON repair context containing verifier stderr,
stdout snippets, git status, and git diff, invokes a configured provider repair
command, parses JSON output, and converts full-file edits or unified diffs into
checked `CodingMissionControlledEdit` values. The existing mission runner then
owns branch restore, mutation, verifier rerun, checkpoint, commit, and PR-ready
evidence.

## Affected Modules

- `crates/tau-runtime/src/autonomous_coding_jobs_runtime.rs`
- `crates/tau-runtime/src/autonomous_coding_repair_runtime.rs`
- `crates/tau-coding-agent/src/bin/tau_autonomous_coding_job.rs`
- `scripts/run/tau-unified.sh`
- `scripts/run/test-tau-unified.sh`
- `scripts/dev/test-autonomous-coding-issue-to-merge.sh`
- `scripts/dev/test-autonomous-coding-gauntlet.sh`
- `docs/guides/autonomous-coding-jobs.md`
- `README.md`

## Risks

- Provider repair command output may be malformed. Mitigation: fail closed,
  persist output hash/artifacts, and block after the attempt budget.
- Unified diff parsing may diverge from prompt-mode `edit_many`. Mitigation:
  support a narrow existing-file subset and reject renames, creates, deletes,
  path escapes, and hunk mismatches.
- Operators may read provider repair as permissionless mutation. Mitigation:
  issue intake still blocks without verifier authority and docs keep the
  authority boundary explicit.

## Verification

- Runtime tests for provider repair success, diff success, malformed output, and
  issue-to-merge no-authority block.
- CLI/script proof for issue-to-merge provider repair and guarded auto-merge.
- `tau-unified` status contract test.
- Rust fmt, clippy, and focused crate tests.
