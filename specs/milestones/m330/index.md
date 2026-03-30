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
- Task: #3636
- Task: #3637
- Task: #3638
- Task: #3639
- Task: #3640
- Task: #3641
- Task: #3642

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
- Keep PR-scoped formatting checks aligned to the changed Rust surface so
  unrelated workspace fmt drift on `master` does not block narrow branches.
- Resolve the remaining package-scoped clippy warning debt that blocks the
  master-targeting `#3631` branch after CI-scope defects are removed.
- Contain deprecation-warning failures inside the intentionally deprecated
  `tau-custom-command` compatibility crate so package-scoped clippy can
  continue past that staged migration boundary.
- Contain deprecation-warning failures inside the intentionally deprecated
  `tau-extensions` compatibility crate so package-scoped clippy can continue
  past that staged migration boundary.
- Contain deprecation-warning failures at the intentional `tau-tools` bridge
  points that still consume the deprecated `tau-extensions` surface during the
  staged migration to `tau-skills`.
- Resolve follow-on real clippy lint debt that becomes visible after the
  compatibility-surface blockers are removed from the package-scoped
  validation path.
- Stabilize the newly visible `tau-coding-agent` package-scoped runtime tests
  so the branch can move beyond validation debt and back to the actual
  oversized-file story.

## Exit Criteria
- `wc -l crates/tau-coding-agent/src/live_rl_runtime.rs` reports `<= 4000`.
- `python3 .github/scripts/oversized_file_guard.py ...` reports `issues=0`
  without a new exemption for `live_rl_runtime.rs`.
- Targeted `tau-coding-agent` live RL runtime selectors pass after the split.
- `specs/3633/spec.md` is `Implemented` with docs-quality evidence.
- `specs/3632/spec.md` is `Implemented` with shallow-history regression
  coverage and focused validation evidence.
- `specs/3636/spec.md` restores PR-scoped formatting validation without
  regressing explicit full-workspace validation modes.
- `specs/3637/spec.md` clears the remaining warning debt in the impacted
  package-scoped Rust validation path.
- `specs/3638/spec.md` keeps `tau-custom-command` lint-clean under
  package-scoped `-D warnings` without changing its runtime behavior.
- `specs/3639/spec.md` keeps `tau-extensions` lint-clean under package-scoped
  `-D warnings` without changing its runtime behavior.
- `specs/3640/spec.md` keeps `tau-tools` lint-clean at the remaining
  extension-compatibility bridge points without muting unrelated lint coverage.
- `specs/3641/spec.md` clears the next real code-quality lint blocker in the
  impacted package-scoped validation path.
- `specs/3642/spec.md` clears the newly exposed `tau-coding-agent`
  package-scoped test failures in the runtime/startup path.
- M330 no longer has hidden blockers unrelated to the `live_rl_runtime.rs`
  split itself.

## Delivery Notes
- Prefer domain-oriented private module extraction over behavior changes.
- Avoid new dependencies, protocol changes, or policy exemptions unless a
  follow-up spec explicitly approves them.
- Keep the work isolated from PR #3628; that PR should continue to document this
  blocker as external debt rather than absorb the split.
