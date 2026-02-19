# M94 - Spacebot G17 Prompt Template Hot-Reload Bridge (Phase 4)

Milestone: GitHub milestone `M94 - Spacebot G17 Prompt Template Hot-Reload Bridge (Phase 4)`

## Objective
Deliver the next bounded `G17` slice from `tasks/spacebot-comparison.md` by coupling workspace startup prompt template edits to live local runtime turns without process restart.

## Scope
- Detect workspace startup template changes during local runtime execution.
- Recompose startup system prompt and apply updates for subsequent turns.
- Preserve fail-closed fallback behavior and deterministic diagnostics when template edits are invalid.
- Add conformance/regression coverage with RED/GREEN and mutation evidence.

## Out of Scope
- Full cross-transport prompt hot-reload (Slack/GitHub/multi-channel runtimes).
- New template language features or variable expansion beyond existing startup prompt composition.
- Dashboard/UI features.

## Issue Hierarchy
- Epic: #2546
- Story: #2547
- Task: #2548
- Subtask: #2549

## Exit Criteria
- Task ACs verified by conformance tests and RED/GREEN evidence.
- `cargo fmt --check`, `cargo clippy -- -D warnings`, scoped tests, mutation in diff, and workspace `cargo test -j 1` pass.
- Live validation run succeeds and issue process logs are updated.
