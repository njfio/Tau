# M330 - Live RL runtime oversized-file closure

Status: Active

## Context
Milestone M330 was created to close the oversized-file policy blocker on
`crates/tau-coding-agent/src/live_rl_runtime.rs` without folding unrelated
repo debt into the same implementation PR. While executing story `#3630`,
two independent blockers surfaced on `master`:

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
- Repair the two broken relative links in March 24 planning docs.
- Restore `Docs Quality` for PRs that touch `docs/**` or `tasks/**`.
- Investigate and correct shallow-history `fast-validate` fallback behavior.

## Exit Criteria
- `specs/3633/spec.md` is `Implemented` with docs-link evidence.
- `specs/3632/spec.md` is accepted and its CI-scope fix is delivered in a
  dedicated change.
- M330 no longer has hidden blockers unrelated to the `live_rl_runtime.rs`
  split itself.
