# M330 - Live RL runtime oversized-file closure

Status: Active

## Context
Milestone M330 was created to close the oversized-file policy blocker on
`crates/tau-coding-agent/src/live_rl_runtime.rs` without folding unrelated
repo debt into the same implementation PR. The repo-wide oversized-file guard
fails on this module because it exceeded the default 4000-line threshold, and
that policy failure blocks the shared quality lane until the file is
decomposed or explicitly exempted.

While executing story `#3630`, two independent blockers surfaced on `master`:

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
- Decompose `crates/tau-coding-agent/src/live_rl_runtime.rs` below the 4000-line
  oversized-file threshold.
- Preserve existing live RL runtime behavior and targeted conformance/regression
  selectors.
- Keep the oversized-file guard green without adding a temporary exemption for
  `live_rl_runtime.rs`.
- Restore deterministic docs workflow behavior for March 24 planning docs and
  generated roadmap-status blocks.
- Restore deterministic validation scoping when CI checks out a shallow PR
  branch and only fetches the PR base commit.

## Exit Criteria
- `wc -l crates/tau-coding-agent/src/live_rl_runtime.rs` reports `<= 4000`.
- `python3 .github/scripts/oversized_file_guard.py ...` reports `issues=0`
  without a new exemption for `live_rl_runtime.rs`.
- Targeted `tau-coding-agent` live RL runtime selectors pass after the split.
- `specs/3633/spec.md` is `Implemented` with docs-quality evidence.
- `specs/3632/spec.md` is `Implemented` with shallow-history regression
  coverage and focused validation evidence.
- M330 no longer has hidden blockers unrelated to the `live_rl_runtime.rs`
  split itself.

## Delivery Notes
- Prefer domain-oriented private module extraction over behavior changes.
- Avoid new dependencies, protocol changes, or policy exemptions unless a
  follow-up spec explicitly approves them.
- Keep the work isolated from PR #3628; that PR should continue to document this
  blocker as external debt rather than absorb the split.
