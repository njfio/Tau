# M83 - Spacebot G16 Hot-Reload Config (Phase 2)

Milestone: GitHub milestone `M83 - Spacebot G16 Hot-Reload Config (Phase 2)`

## Objective
Deliver the second bounded `G16` slice from `tasks/spacebot-comparison.md` by replacing poll-only heartbeat hot-reload with notify-driven profile-policy TOML watching and lock-free active config swaps.

## Scope
- Add `notify` watcher flow for heartbeat profile-policy TOML updates.
- Add `ArcSwap`-backed active runtime heartbeat config storage for lock-free reads during scheduler cycles.
- Parse + validate profile-policy TOML changes and apply valid updates atomically.
- Preserve fail-closed behavior for invalid updates with deterministic diagnostics/reason codes.
- Add conformance/regression tests for update/no-change/invalid-change paths.

## Out of Scope
- Full profile-wide runtime hot-reload across all modules.
- Prompt-template file watch behavior (`G17` follow-on scope).
- New process-type routing model dispatch (`G15`/`G1` follow-on scope).

## Issue Hierarchy
- Epic: #2485
- Story: #2486
- Task: #2487
- Subtask: #2488

## Exit Criteria
- ACs for #2487 are verified by conformance tests and RED/GREEN evidence from #2488.
- `cargo fmt --check`, `cargo clippy -p tau-runtime -- -D warnings`, and scoped `tau-runtime` tests pass.
- M83 issues are closed with `status:done` and specs marked `Implemented`.
