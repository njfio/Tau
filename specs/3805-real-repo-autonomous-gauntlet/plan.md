# Plan: Tau-Internal Autonomous Coding Gauntlet

## Approach

Add one shell-level benchmark script that uses the already-landed autonomous
coding job CLI and durable runtime. The script creates a temporary worktree,
seeds small Tau-local fixtures, runs each issue-to-merge scenario, and records a
JSONL result plus a final JSON report. Successful cases use bounded fake provider
repair scripts so the benchmark is deterministic and does not require live
credentials.

## Affected Modules

- `scripts/dev/test-real-repo-autonomous-coding-gauntlet.sh`: new gauntlet
  runner.
- `README.md`: evidence table entry.
- `docs/guides/autonomous-coding-jobs.md`: operator proof guidance.
- `specs/3805-real-repo-autonomous-gauntlet/`: binding spec artifacts.

## Risks and Mitigations

- Risk: temporary branches could collide with developer branches.
  Mitigation: use a run-scoped `codex/tmp-real-repo-gauntlet-*` branch prefix and
  a process-specific base branch.
- Risk: the script could mutate the active checkout.
  Mitigation: all benchmark fixtures are created in a temporary detached
  worktree; cleanup removes only run-scoped worktree/branches.
- Risk: provider behavior could make results flaky.
  Mitigation: use deterministic fake provider adapters for this benchmark and
  keep live-provider proof in separate opt-in scripts.
- Risk: verifier commands could depend on shell parsing that Tau does not
  provide.
  Mitigation: use argv-safe verifier commands and small fixture scripts where
  shell behavior is needed.

## Interfaces

- Input: current repository branch/commit.
- Output: `/tmp/tau-real-repo-autonomous-coding-gauntlet/real-repo-gauntlet-report.json`.
- CLI surface: `tau-autonomous-coding-job issue-to-merge`, `status`, and
  `mark-blocked`.

## Rollback

Remove the script, README/guide references, and this spec directory. The runtime
code path remains unchanged by this benchmark.
