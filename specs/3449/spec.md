# Spec: Issue #3449 - M298 workspace artifact hygiene and ignore policy

Status: Implemented

## Problem Statement
The local workspace accumulates large transient outputs from verification runs (`artifacts/`, `target-fast-*`, `mutants.out*`, cache directories), which obscures reviewable diffs and increases risk of accidental commits.

## Scope
In scope:
- Define ignore policy for deterministic local-generated artifacts.
- Update `.gitignore` with explicit patterns for transient outputs.
- Remove current transient artifacts from workspace.

Out of scope:
- Deleting hand-authored docs/spec/task files.
- Changing CI behavior.

## Acceptance Criteria
### AC-1 Ignore policy covers known transient outputs
Given local verification/build workflows,
when generated artifacts are produced,
then `.gitignore` prevents them from appearing as untracked noise.

### AC-2 Workspace cleanup removes current transient outputs
Given current local workspace state,
when cleanup commands run,
then known transient directories/files are removed while authored source/spec files remain.

### AC-3 Cleanup evidence is explicit and reproducible
Given issue verification,
when status and checks are reviewed,
then commands/results demonstrate ignore policy and cleaned workspace behavior.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Conformance | updated `.gitignore` | run `git status --short` after transient generation | transient paths are ignored |
| C-02 | AC-2 | Functional | workspace with known transient dirs | run cleanup command set | transient dirs removed; authored files remain |
| C-03 | AC-3 | Regression | post-cleanup workspace | run status/inspection commands | evidence shows reduced noise and no unintended tracked deletions |

## Success Metrics / Observable Signals
- `.gitignore` contains explicit transient-output patterns.
- `git status --short` no longer lists cleaned transient outputs.
- No authored spec/docs/code files are deleted by cleanup.

## Implementation Verification (2026-02-24)
| AC | Result | Verification |
| --- | --- | --- |
| AC-1 | ✅ | `.gitignore` updated with transient-output patterns for artifacts, target-fast/target-m295+/target-m296+, mutants, and local cache paths |
| AC-2 | ✅ | Scoped cleanup commands removed known transient paths (artifacts/mutants/cache/target output directories) |
| AC-3 | ✅ | `git status --short --branch` now shows reduced noise and retained authored files (PRD/task/spec docs remain) |
