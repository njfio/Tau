# M251 - Self-Improvement APO Live Integration (Phase 3A)

Status: Completed

## Context
Review #36 identifies APO as implemented but not connected to the live RL runtime. Tau currently computes intrinsic rewards and PPO updates, but prompt self-optimization is not executed from live rollout traces.

## Scope
- Wire APO execution into `live_rl_runtime` update cycles.
- Build reward-informed APO datasets from persisted live decision spans.
- Gate prompt adoption with deterministic significance checks before persisting resource updates.

## Linked Issues
- Epic: #3294
- Story: #3295
- Task: #3296
- Task: #3298
- Task: #3300
- Task: #3302

## Closeout
- Milestone closed on GitHub with `open_issues=0` and `closed_issues=6`.
- Final completion delivered via `#3303` (issues `#3300` + `#3302`).

## Success Signals
- `cargo test -p tau-coding-agent spec_c07_functional_live_optimizer_runs_apo_and_persists_prompt_resources`
- `cargo test -p tau-coding-agent spec_c08_regression_live_apo_skips_adoption_without_significant_improvement`
- `cargo test -p tau-coding-agent spec_c02_functional_optimizer_runs_on_update_interval`
- `cargo fmt --check`
- `cargo clippy -p tau-coding-agent --no-deps -- -D warnings`
