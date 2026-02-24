# Tasks: Issue #3449 - M298 workspace artifact hygiene and ignore policy

Status: Implemented

## Ordered Tasks
1. [x] T1 (RED, Conformance): capture untracked transient artifact baseline from `git status --short --branch`.
2. [x] T2 (GREEN, Docs/Config): update `.gitignore` with specific transient-output patterns.
3. [x] T3 (GREEN, Functional): remove known transient artifacts from the workspace.
4. [x] T4 (VERIFY, Regression): re-run `git status --short --branch` and verify authored files remain.
5. [x] T5 (VERIFY): set spec/tasks status to `Implemented` once ACs pass.

## Verification Evidence (2026-02-24)
### RED
- `git status --short --branch`
- Result (pre-cleanup): large untracked transient surface including `artifacts/`, `mutants.out*`, `target-fast-*`, `target-m295-*`, `target-m296-*`, cache directories, and local executable artifacts.

### GREEN
- `.gitignore` updated with explicit transient-output patterns.
- Scoped cleanup command sets executed for known generated paths.
- Result: transient artifact footprint reduced and ignored for future runs.

### REGRESSION
- `git status --short --branch`
- Result (post-cleanup): transient generated paths no longer dominate workspace state; authored PRD/spec/task docs remain present.

## Test Tier Matrix
| Tier | ✅/❌/N/A | Tests / Evidence | N/A Why |
|---|---|---|---|
| Unit | N/A | No runtime code change | Hygiene/config only |
| Property | N/A | No invariants/parser changes | Hygiene/config only |
| Contract/DbC | N/A | No public API contract changes | Hygiene/config only |
| Snapshot | N/A | No snapshot behavior surface | Hygiene/config only |
| Functional | ✅ | Scoped cleanup execution and path checks | |
| Conformance | ✅ | Baseline + post-cleanup status checks | |
| Integration | N/A | No cross-service behavior changed | Hygiene/config only |
| Fuzz | N/A | No untrusted-input runtime path changed | Hygiene/config only |
| Mutation | N/A | No production code paths changed | Hygiene/config only |
| Regression | ✅ | Post-cleanup status verification | |
| Performance | N/A | No performance-sensitive logic changed | Hygiene/config only |
