# Spec: Issue #3188 - resync stale claims in tasks/whats-missing.md

Status: Implemented

## Problem Statement
`tasks/whats-missing.md` reports multiple capabilities as missing even though they are now implemented in the repository. This creates a false roadmap signal and can redirect engineering effort away from true gaps.

## Scope
In scope:
- Create/update `tasks/whats-missing.md` as a tracked, current-state inventory.
- Replace stale claims with verified status for implemented items.
- Keep only unresolved gaps and mark them with concrete evidence.
- Add script-level conformance checks that fail when stale markers reappear.

Out of scope:
- Implementing unresolved product/runtime gaps.
- Changing runtime behavior outside documentation + verification scripts.

## Acceptance Criteria
### AC-1 Implemented capabilities are no longer listed as missing
Given the current repository state,
when `tasks/whats-missing.md` is read,
then claims that are already implemented (cost tracking, OpenRouter provider, Postgres backend, Docker/Homebrew/completions, fuzzing, log rotation) are represented as implemented/resolved and not as missing.

### AC-2 Remaining missing items are explicit and evidence-based
Given unresolved gaps,
when `tasks/whats-missing.md` lists them,
then each remaining gap includes concise evidence and current impact language.

### AC-3 Conformance guard prevents stale-claim regressions
Given the refreshed report,
when `scripts/dev/test-whats-missing.sh` runs,
then it validates required markers and fails on stale markers.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
|---|---|---|---|---|---|
| C-01 | AC-1 | Functional/Conformance | refreshed report | run conformance script | stale missing-claims markers fail if present |
| C-02 | AC-2 | Functional/Conformance | refreshed report | run conformance script | unresolved gaps markers must be present |
| C-03 | AC-3 | Functional/Conformance | script + report in repo | execute script | script exits 0 only when report matches expected markers |

## Success Metrics / Observable Signals
- `scripts/dev/test-whats-missing.sh`
- `cargo fmt --check`
- `cargo clippy -- -D warnings`
