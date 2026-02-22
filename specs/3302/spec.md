# Spec: Issue #3302 - complete Review #37 Phase-4 depth (curriculum + meta-cognition)

Status: Implemented

## Problem Statement
`tasks/review-37.md` reports that Phase-4 remains partially delivered after #3300. Baseline category, trend, and help signals exist, but Tau still lacks long-horizon curriculum aggregation, progressive difficulty scheduling, calibration-curve reporting surfaces, and dashboard-visible learning alerts.

## Scope
In scope:
- Expand task-category normalization into a richer canonical taxonomy used by live RL spans and curriculum analytics.
- Persist long-horizon per-category curriculum aggregates (samples, reward, success, calibration, trend, difficulty) to latest resources/status artifacts.
- Add progressive difficulty weighting to APO sample capping so harder categories receive stronger selection priority under sample caps.
- Persist confidence calibration-curve bins and derived alert signals.
- Surface learning alerts through gateway dashboard snapshot/command-center alert feed using existing dashboard status artifacts.

Out of scope:
- New frontend routes/pages.
- New external dependencies.
- Changes to PPO/GAE/APO core algorithms outside runtime scheduling inputs.

## Acceptance Criteria
### AC-1 Canonical taxonomy + long-horizon aggregates are persisted
Given succeeded live RL rollouts across categories,
when runtime finalizes runs,
then latest resources/status artifacts include canonical-category aggregate entries with samples, success rate, mean reward, trend, calibration error, and difficulty score.

### AC-2 APO selection uses progressive difficulty weighting
Given APO samples exceed cap and category difficulty weights differ,
when runtime selects capped APO samples,
then harder categories are prioritized in the capped set and focus-category diagnostics reflect the hardest category.

### AC-3 Calibration curves + learning alerts are emitted
Given historical outcomes with varied predicted-success bins,
when runtime persists curriculum/meta-cognition summaries,
then it emits calibration-curve bins, global calibration error, and deterministic learning alerts (e.g., regressing category / poor calibration).

### AC-4 Dashboard surfaces learning alerts
Given training status includes live learning alerts,
when gateway collects dashboard snapshot,
then command-center alert feed includes those learning alerts with preserved severity/code/message.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
|---|---|---|---|---|---|
| C-01 | AC-1 | Functional/Conformance | mixed succeeded live rollouts | runtime finalizes + persists summaries | resources/status contain canonical category aggregate blocks |
| C-02 | AC-2 | Regression/Conformance | capped APO sample pool with unequal difficulty weights | sample selection runs | selected set/focus category prioritizes hardest category |
| C-03 | AC-3 | Regression/Conformance | outcomes spanning confidence bins with regressing reward trend | summary persistence runs | calibration bins + alert set populated deterministically |
| C-04 | AC-4 | Integration/Conformance | status artifact with live learning alerts | gateway snapshot collection runs | command-center alert feed contains learning alerts |

## Success Metrics / Observable Signals
- `cargo test -p tau-coding-agent --bin tau-coding-agent live_rl_runtime::tests::spec_c18_regression_live_curriculum_aggregates_persisted_to_resources`
- `cargo test -p tau-coding-agent --bin tau-coding-agent live_rl_runtime::tests::spec_c19_regression_live_apo_progressive_difficulty_prioritizes_harder_category`
- `cargo test -p tau-coding-agent --bin tau-coding-agent live_rl_runtime::tests::spec_c20_regression_live_calibration_curve_and_alerts_are_persisted`
- `cargo test -p tau-gateway dashboard_status::tests::spec_c21_integration_dashboard_snapshot_includes_live_learning_alerts`
- `cargo fmt --check`
- `cargo clippy -p tau-coding-agent -p tau-gateway --no-deps -- -D warnings`
