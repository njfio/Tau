Status: Implemented

# Spec 3805: Tau-Internal Autonomous Coding Gauntlet

## Problem

The autonomous coding loop has deterministic fixture-repo coverage for provider
repair, issue-to-merge orchestration, and guarded merge behavior. That is useful,
but it does not prove the harness against a Tau-shaped repository checkout with
real branch, verifier, commit, stale-lease, and blocked-state behavior. Tau needs
a repeatable real-repo gauntlet that exercises the product loop in a temporary
worktree without mutating the developer checkout.

## Scope

In scope:
- A script that creates a temporary Tau worktree and seeds small benchmark
  fixtures inside it.
- Successful issue-to-merge cases for docs-only, single-file bug, multi-file
  bug, failing-test repair, CLI flag change, and flaky verifier behavior.
- Fail-closed coverage for malformed provider output.
- Operator classification coverage for a simulated crash/stale lease.
- A JSON report that records per-case pass/fail status over time.

Out of scope:
- Live third-party provider calls.
- Protected-branch bypass or admin merge.
- Large arbitrary issue solving without verifier and edit/provider authority.
- Keeping the temporary benchmark fixtures in the main repository tree.

## Acceptance Criteria

### AC-1: The gauntlet runs against a Tau worktree

Given a clean or dirty developer checkout
When the gauntlet starts
Then it must create an isolated temporary worktree from the current branch
And it must clean up temporary branches/worktrees after execution.

### AC-2: The gauntlet covers representative successful coding cases

Given bounded provider repair authority
When the gauntlet runs docs-only, single-file, multi-file, failing-test repair,
CLI flag, and flaky verifier cases
Then each case must reach PR-ready evidence through the durable issue-to-merge
loop.

### AC-3: Malformed provider output fails closed

Given provider output that cannot produce an applicable edit
When the gauntlet runs the malformed-provider case
Then Tau must block the job, record rejected provider repair evidence, and avoid
committing the malformed repair.

### AC-4: Crash/stale-lease state is operator-actionable

Given a durable job record with an expired lease
When the gauntlet requests job status
Then Tau must classify the job as `stale_lease`, mark it recoverable, and expose
an operator recovery command.

### AC-5: Results are stored as benchmark evidence

Given the gauntlet completes
When it writes its report
Then the report must list every case, pass/fail state, detail artifact path, and
suite totals.

## Conformance Cases

- C-01 maps AC-1/AC-2: `docs-only` repairs a markdown fixture and reaches
  PR-ready evidence.
- C-02 maps AC-1/AC-2: `single-file-bug` repairs one data fixture and reaches
  PR-ready evidence.
- C-03 maps AC-1/AC-2: `multi-file-bug` applies a multi-file provider repair and
  reaches PR-ready evidence.
- C-04 maps AC-1/AC-2: `failing-test-repair` fixes a failing verifier script and
  reaches PR-ready evidence.
- C-05 maps AC-1/AC-2: `cli-flag-change` adds a CLI flag path and reaches
  PR-ready evidence.
- C-06 maps AC-1/AC-2: `flaky-verifier` handles a first-run verifier failure,
  applies repair, reruns, and reaches PR-ready evidence.
- C-07 maps AC-3: `malformed-provider-patch` records rejected repair evidence and
  blocks safely.
- C-08 maps AC-4/AC-5: `crash-mid-run` classifies stale lease state and the final
  report records eight passing cases.

## Success Signals

- `CARGO_TARGET_DIR=/tmp/rust_pi-3805-target scripts/dev/test-real-repo-autonomous-coding-gauntlet.sh`
  exits zero and prints `real_repo_autonomous_coding_gauntlet=pass`.
- `/tmp/tau-real-repo-autonomous-coding-gauntlet/real-repo-gauntlet-report.json`
  reports eight passed cases and zero failed cases.
