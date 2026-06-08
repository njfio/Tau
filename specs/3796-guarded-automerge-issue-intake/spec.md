# Issue #3796: Guarded Auto-Merge And Arbitrary Issue Intake

Status: Implemented
GitHub Issue: #3796
Priority: P1
Milestone: M334 - Tau Ralph loop supervisor architecture

## Problem

Durable autonomous coding jobs can reach PR-ready state, but the product loop
still stops at a manual handoff. Operators need Tau to request auto-merge when
the repository's protected-branch rules allow it, and they need Tau to ingest
arbitrary issue context even when verifier/edit authority has not been granted.

The unsafe version of this request would bypass protected branches or mutate
repositories without authority. That is out of scope. The product behavior must
honor GitHub protections and block with an actionable authority plan when
required verifier/edit authority is missing.

## Scope

In scope:
- Add policy-gated auto-merge request support to autonomous coding jobs.
- Require PR-ready state, explicit policy, a PR URL, and GitHub auth before a
  merge request is attempted.
- Use `gh pr merge <url> --auto --<method>` without admin/protection bypass.
- Persist auto-merge evidence and expose it in job status.
- Add issue intake without verifier/edit authority that persists an authority
  plan and status instead of mutating code.
- Add CLI and integration coverage for blocked and successful guarded paths.

Out of scope:
- Bypassing protected branch rules, required reviews, or required checks.
- Blind arbitrary repo mutation without verifier/edit authority.
- Automatically merging directly to protected branches with admin override.

## Acceptance Criteria

### AC-1: Auto-Merge Requires Explicit Authority

Given a PR-ready autonomous coding job has a PR URL, GitHub auth, and
`allow_auto_merge=true`, when auto-merge is requested, then Tau invokes a GitHub
auto-merge command that honors branch protection and persists merge evidence.

### AC-2: Missing Authority Blocks Safely

Given a job is not PR-ready, has no PR URL, lacks GitHub auth, or does not have
auto-merge policy enabled, when auto-merge is requested, then Tau does not run a
merge command and records an operator-visible blocked reason.

### AC-3: Protected Branches Are Not Bypassed

Given auto-merge is requested, when the command is assembled, then it includes
`--auto` and a merge method but never includes `--admin` or another branch
protection bypass flag.

### AC-4: Arbitrary Issue Intake Without Authority Produces A Plan

Given an arbitrary issue URL/body is ingested without verifier or edit
authority, when intake runs, then Tau persists a durable issue-intake plan with
required verifier/edit authority and marks the intake blocked without changing
the repository.

## Conformance Cases

| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Functional | PR-ready job with PR URL, auth, and policy enabled | auto-merge request runs | fake `gh` receives `pr merge <url> --auto --squash` and status records requested |
| C-02 | AC-2 | Regression | PR-ready job without PR URL or policy | auto-merge request runs | command is not executed and status records blocked reason |
| C-03 | AC-3 | Conformance | fake `gh` captures argv | auto-merge request runs | argv has no `--admin` |
| C-04 | AC-4 | Functional | issue URL/body without verifier/edit authority | issue intake runs | authority plan exists and repo files are unchanged |

## Success Signals

- Focused `tau-runtime` tests cover C-01..C-04.
- CLI integration script proves guarded merge and no-authority issue intake over
  disposable fixtures.
- Existing autonomous coding job loop proof continues to pass.
