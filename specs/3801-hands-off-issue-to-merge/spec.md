# Spec 3801: Hands-Off Issue-To-Merge Orchestration

Status: Reviewed
Priority: P1
Milestone: M334 - Tau Ralph loop supervisor architecture

## Problem

Tau has the building blocks for autonomous coding jobs: issue intake, durable job
records, verifier-gated mission execution, PR-ready bundles, and guarded
auto-merge requests. The operator experience is still too manual because those
steps are separate commands. A hands-off issue-to-merge loop needs one product
surface that either runs the full authorized loop or blocks before mutation with
an explicit authority plan.

## Scope

In scope:
- Add one runtime orchestration request for issue-to-merge.
- Add one CLI command that runs intake -> submit -> run -> PR-ready -> optional
  auto-merge request.
- Reuse `CodingMissionRunner`, autonomous coding job records, issue-intake
  records, and existing protected-branch-safe auto-merge behavior.
- Block without mutating when verifier commands or edit authority are missing.
- Persist status and evidence through the existing job/intake files.

Out of scope:
- Bypassing protected branches, required checks, or required reviews.
- Mutating arbitrary repositories without verifier/edit authority.
- Provider-generated edit planning inside the durable job runtime.
- Large dashboard command-center UX polish.

## Acceptance Criteria

AC-1: Given an issue-to-merge request includes verifier commands and controlled
edits, when the orchestration runs, then Tau submits a durable job, runs the
mission to `pr_ready`, prepares PR-ready evidence, and returns one outcome with
the job/run status.

AC-2: Given the same request has draft PR mode, GitHub auth, and fake `gh`
available, when the orchestration runs, then Tau creates a draft PR through the
existing PR-ready bundle path and records the PR URL.

AC-3: Given auto-merge is explicitly allowed and the job has a PR URL, when the
orchestration reaches PR-ready, then Tau requests normal GitHub auto-merge with
`gh pr merge --auto` and never includes admin override flags.

AC-4: Given verifier commands or edit authority are missing, when
issue-to-merge runs, then Tau persists an issue-intake blocked authority plan and
does not change repository files.

## Conformance Cases

C-01 maps AC-1: disposable repo with failing verifier plus controlled edits
runs through `issue-to-merge` and reaches `pr_ready`.

C-02 maps AC-2 and AC-3: fake `gh` captures draft PR creation and auto-merge
arguments; outcome records `auto_merge_requested` and no `--admin` argument.

C-03 maps AC-4: issue-to-merge without edits persists
`issue_intake_authority_required`, leaves repo files unchanged, and does not
create a job record.

## Success Signals

- `cargo test -p tau-runtime autonomous_coding_jobs_runtime -- --test-threads=1`
- `cargo test -p tau-coding-agent --bin tau_autonomous_coding_job -- --test-threads=1`
- `scripts/dev/test-autonomous-coding-issue-to-merge.sh`
