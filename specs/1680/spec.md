# Issue 1680 Spec

Status: Accepted

Issue: `#1680`  
Milestone: `#21`  
Parent: `#1635`

## Problem Statement

`crates/tau-github-issues-runtime/src/github_issues_runtime.rs` was previously
decomposed, but this subtask is still open without issue-scoped spec artifacts
and explicit conformance evidence for the runtime-domain split contract.

## Scope

In scope:

- add `specs/1680/{spec,plan,tasks}.md`
- add a contract harness for GitHub Issues runtime-domain split verification
- verify the runtime file remains under maintainability threshold
- verify extracted runtime-domain modules are wired and present
- run targeted runtime tests and quality checks

Out of scope:

- changing bridge behavior
- introducing new flags or protocol changes
- broad runtime refactors outside the existing split boundaries

## Acceptance Criteria

AC-1 (line budget):
Given `crates/tau-github-issues-runtime/src/github_issues_runtime.rs`,
when conformance checks run,
then line count is below `4000`.

AC-2 (domain extraction):
Given runtime split domains (poll/api/client helpers, command/render helpers,
run-task orchestration, session/state persistence, prompt execution),
when conformance checks run,
then module declarations and extracted files are present under
`crates/tau-github-issues-runtime/src/github_issues_runtime/`.

AC-3 (behavior parity):
Given existing runtime tests,
when targeted tests run,
then GitHub Issues bridge behavior remains green.

AC-4 (verification):
Given issue-scope checks,
when run,
then harness + targeted tests + roadmap/fmt/clippy pass.

## Conformance Cases

| Case | Maps To | Tier | Given / When / Then |
| --- | --- | --- | --- |
| C-01 | AC-1 | Functional | Given split runtime file, when harness runs, then `github_issues_runtime.rs < 4000` lines. |
| C-02 | AC-2 | Functional | Given module tree, when harness runs, then required module markers/files are present. |
| C-03 | AC-3 | Regression | Given targeted runtime tests, when run, then they pass unchanged. |
| C-04 | AC-4 | Integration | Given issue commands, when run, then harness + tests + roadmap/fmt/clippy are green. |

## Success Metrics

- runtime file remains under threshold with domain modules intact
- extracted runtime-domain boundaries are verifiable via harness
- issue closure includes explicit spec-driven evidence
