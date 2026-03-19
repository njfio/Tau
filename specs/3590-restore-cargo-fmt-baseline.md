# 3590 Restore Cargo Fmt Baseline

## Objective
Restore the repository formatting baseline for the tau-tui interactive files currently failing the required Quality CI gate.

## Inputs/Outputs
- Input: current `origin/master` formatting state and `cargo fmt --check --all` output
- Output: rustfmt-normalized source in the affected tau-tui files only

## Boundaries/Non-goals
- Do not change runtime behavior, control flow, or UI semantics.
- Do not edit files beyond the rustfmt-affected files and this spec unless the formatter proves additional files are required.
- Do not mix in functional TUI work.

## Failure modes
- `cargo fmt --check --all` still reports formatting diffs after the change.
- Formatting touches unexpected files outside the affected scope.
- Any non-formatting diff appears in the touched tau-tui files.

## Acceptance criteria
- [ ] `cargo fmt --check --all` exits 0 from a clean checkout after the change.
- [ ] The diff is limited to rustfmt-only changes in `crates/tau-tui/src/interactive/ui.rs` and `crates/tau-tui/src/main.rs`, unless `cargo fmt --check --all` requires additional files.
- [ ] No behavior changes are introduced.

## Files to touch
- `specs/3590-restore-cargo-fmt-baseline.md`
- `crates/tau-agent-core/src/agent_channel.rs`
- `crates/tau-agent-core/src/circuit_breaker.rs`
- `crates/tau-agent-core/src/context_ranking.rs`
- `crates/tau-agent-core/src/failure_detector.rs`
- `crates/tau-agent-core/src/lib.rs`
- `crates/tau-agent-core/src/metrics.rs`
- `crates/tau-ai/tests/provider_live_integration.rs`
- `crates/tau-memory/src/runtime/ranking.rs`
- `crates/tau-orchestrator/src/plan.rs`
- `crates/tau-orchestrator/src/plan_executor.rs`
- `crates/tau-tools/src/tools.rs`
- `crates/tau-tui/src/interactive/app.rs`
- `crates/tau-tui/src/interactive/mod.rs`
- `crates/tau-tui/src/interactive/ui.rs`
- `crates/tau-tui/src/main.rs`

## Error semantics
- Fail loud if `cargo fmt --check --all` still reports diffs.
- Do not hand-format selectively in a way that diverges from rustfmt.

## Test plan
- Red: run `cargo fmt --check --all` and confirm formatting diffs are reported.
- Green: run rustfmt on the affected files.
- Verification: rerun `cargo fmt --check --all` and confirm zero exit.
- Verification: inspect the diff to confirm formatting-only changes.
