# M330 - Live RL runtime oversized-file closure

Status: Active

## Context
Milestone M330 was created to close the oversized-file policy blocker on
`crates/tau-coding-agent/src/live_rl_runtime.rs` without folding unrelated
repo debt into the same implementation PR. While executing story `#3630`, two
independent blockers surfaced on `master`:

1. Broken relative links and stale generated roadmap-status docs in the March
   24 planning/docs workflow path (`#3633`).
2. `fast-validate` losing scoped validation under shallow PR history because
   CI checks out the PR branch with insufficient ancestry for `base...HEAD`
   diffing (`#3632`).

Those blockers are tracked as separate tasks so the oversized-file work can
remain narrow and auditable.

## Issue Hierarchy
- Epic: #3629
- Story: #3630
- Task: #3632
- Task: #3633

## Scope
- Restore deterministic docs workflow behavior for March 24 planning docs and
  generated roadmap-status blocks.
- Restore deterministic validation scoping when CI checks out a shallow PR
  branch and only fetches the PR base commit.

## Exit Criteria
- `specs/3633/spec.md` is `Implemented` with docs-quality evidence.
- `specs/3632/spec.md` is `Implemented` with shallow-history regression
  coverage and focused validation evidence.
- PR `#3631` no longer appears red because of unrelated docs/validation
  blockers outside the `live_rl_runtime.rs` split itself.
