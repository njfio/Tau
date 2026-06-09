# Spec: Issue #3802 - Provider Repair Inside Durable Autonomous Coding Jobs

Status: Implemented

## Problem

Tau has a real issue-to-merge path, but durable jobs still require supplied
controlled edits. The missing product loop is verifier failure -> provider
repair proposal -> checked patch/edit application -> verifier rerun -> PR-ready
or blocked evidence.

## Scope

In scope:
- Add provider repair policy to durable autonomous coding jobs.
- Persist provider repair attempts, event log, heartbeat, lease, and checkpoint
  status for operator recovery.
- Allow `issue-to-merge` to proceed without `--edit` when a verifier and provider
  repair adapter are configured.
- Accept provider repair output as full-file edits or existing-file unified
  diffs, and preflight all paths/hunks before applying through the mission
  runner.
- Surface job repair/recovery/PR state through `tau-unified status`.
- Add deterministic gauntlet coverage for success, malformed provider output,
  no-authority blocking, and safe PR behavior.

Out of scope:
- Bypassing branch protection or using `gh pr merge --admin`.
- Mutating without a verifier plan.
- Claiming arbitrary issue solving when neither controlled edits nor provider
  repair authority is configured.
- Replacing the richer prompt-mode `edit_many` tool.

## Acceptance Criteria

AC-1: Given an autonomous coding job has verifier commands and a provider repair
adapter, when the first verifier fails and no controlled edit was supplied, then
Tau writes provider repair context, invokes the adapter, parses its edit/diff
output, applies the resulting checked edit set, reruns verifiers, and reaches
`pr_ready`.

AC-2: Given provider repair output is malformed or invalid, when attempts are
exhausted, then Tau blocks the job with persisted repair evidence and does not
commit.

AC-3: Given an issue-to-merge request lacks verifier commands, when it runs, then
Tau persists an intake classification and does not mutate the repository.

AC-4: Given an issue-to-merge request has verifier commands and provider repair
authority but no manual edits, when it runs, then Tau can produce PR-ready or
draft-PR evidence through the durable job runtime.

AC-5: Given `tau-unified status` has an autonomous coding status file, when it
renders, then operators see job id, status, verifier summary, repair status,
event log path, heartbeat/lease, PR URL/state, and auto-merge state.

AC-6: Given auto-merge is enabled, when Tau requests merge, then it uses normal
`gh pr merge --auto` flags only and never emits `--admin`.

## Conformance Cases

C-01 maps AC-1/AC-4: disposable issue-to-merge run with no `--edit`, fake
provider repair command, RED verifier, provider edit, GREEN verifier, commit,
and PR-ready/draft evidence.

C-02 maps AC-1: provider repair emits a unified diff for an existing file; Tau
preflights the hunk and reaches PR-ready.

C-03 maps AC-2: provider repair emits malformed output; Tau records failed
attempt evidence and blocks without commit.

C-04 maps AC-3: no verifier command persists issue intake classification and
leaves the repo unchanged.

C-05 maps AC-5: `tau-unified status` prints autonomous coding fields from the
latest status snapshot.

C-06 maps AC-6: issue-to-merge draft PR plus auto-merge fake `gh` proof includes
`--auto` and does not include `--admin`.

## Success Signals

- Focused `tau-runtime` autonomous coding job tests pass.
- `tau_autonomous_coding_job` CLI compiles and the issue-to-merge script passes.
- `tau-unified` status contract test passes.
- New gauntlet script emits a pass marker.
