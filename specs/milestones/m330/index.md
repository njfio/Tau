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

1. Broken relative links in March 24 planning documents.
2. CI validation scope expansion when shallow PR history prevents merge-base
   calculation.

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
- Repair the two broken relative links in March 24 planning docs.
- Restore `Docs Quality` for PRs that touch `docs/**` or `tasks/**`.
- Investigate and correct shallow-history `fast-validate` fallback behavior.

## Exit Criteria
- `wc -l crates/tau-coding-agent/src/live_rl_runtime.rs` reports `<= 4000`.
- `python3 .github/scripts/oversized_file_guard.py ...` reports `issues=0`
  without a new exemption for `live_rl_runtime.rs`.
- Targeted `tau-coding-agent` live RL runtime selectors pass after the split.
- `specs/3633/spec.md` is `Implemented` with docs-link evidence.
- `specs/3632/spec.md` is accepted and its CI-scope fix is delivered in a
  dedicated change.
- M330 no longer has hidden blockers unrelated to the `live_rl_runtime.rs`
  split itself.

## Delivery Notes
- Prefer domain-oriented private module extraction over behavior changes.
- Avoid new dependencies, protocol changes, or policy exemptions unless a
  follow-up spec explicitly approves them.
- Keep the work isolated from PR #3628; that PR should continue to document this
  blocker as external debt rather than absorb the split.
