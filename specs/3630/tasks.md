# Tasks: Issue #3630 - Split live_rl_runtime below oversized-file threshold

1. [ ] T1 (RED): capture current blocking evidence:
   - `wc -l crates/tau-coding-agent/src/live_rl_runtime.rs`
   - oversized-file guard failure against repo policy inputs.
2. [ ] T2 (PLAN): map extraction seams and decide the first private helper/module
   moves that reduce root-file size without changing behavior.
3. [ ] T3 (GREEN): extract one or more cohesive private modules from
   `live_rl_runtime.rs` until the root file is `<= 4000` lines.
4. [ ] T4 (REGRESSION): rerun targeted live RL runtime selectors:
   - `spec_c02_functional_optimizer_runs_on_update_interval`
   - `spec_c18_regression_live_curriculum_aggregates_persisted_to_resources`
   - `spec_c21_regression_live_apo_algorithm_failure_reports_deterministic_reason_code`
5. [ ] T5 (VERIFY): rerun oversized-file guard plus scoped formatting/lint checks
   and capture the final evidence in the story issue/PR.
