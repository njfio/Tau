# Spec: Issue #3300 - close Review #37 curriculum/meta-cognition gaps in live RL runtime

Status: Implemented

## Problem Statement
`tasks/review-37.md` reports two unresolved self-improvement gaps for live RL runtime behavior: curriculum weighting and meta-cognition/confidence calibration. It also marks OpenTelemetry export as not started even though #2616 shipped runtime/gateway OTel export paths. We need deterministic runtime coverage for curriculum focus + confidence signals and correct the stale review artifact.

## Scope
In scope:
- Add deterministic task-category attribution for live decision spans.
- Add confidence/meta-cognition attributes on persisted live spans:
  - predicted success probability
  - actual success outcome
  - confidence calibration error
  - ask-for-help recommendation
  - learning trend marker per category
- Change APO sample capping from pure recency to curriculum-weighted selection that prioritizes weaker categories.
- Surface curriculum focus category in APO diagnostics.
- Correct `tasks/review-37.md` status text for OpenTelemetry and remaining missing work.

Out of scope:
- New OpenTelemetry dependencies/exporters (already delivered under #2616).
- New external APIs/UI/dashboard endpoints.
- Replacing PPO/GAE/APO algorithm implementations.

## Acceptance Criteria
### AC-1 live decision spans include confidence/meta-cognition telemetry
Given a completed live run,
when the final decision span is persisted,
then it includes deterministic category + confidence/meta-cognition fields (`task_category`, `predicted_success_probability`, `actual_success`, `confidence_calibration_error`, `ask_for_help_recommended`, `learning_trend`).

### AC-2 APO capping uses curriculum weighting
Given APO samples exceed `apo_max_samples` and categories have different historical reward means,
when live APO update builds its sample window,
then weak categories are prioritized into the capped window and APO diagnostics expose the chosen curriculum focus category.

### AC-3 learning trend state is emitted per category
Given sufficient historical category outcomes,
when a new run in that category is persisted,
then runtime emits a trend classification (`improving|plateau|regressing|insufficient_data`) based on deterministic recent-vs-prior reward windows.

### AC-4 Review #37 no longer reports stale OpenTelemetry status
Given `tasks/review-37.md`,
when reviewed after implementation,
then OpenTelemetry is represented as already implemented with evidence pointers and "not started" is removed for that phase.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
|---|---|---|---|---|---|
| C-01 | AC-1 | Functional/Conformance | live run with user + assistant messages | agent end persists span | span contains category + confidence/meta-cognition keys |
| C-02 | AC-2 | Regression/Conformance | mixed-category APO sample pool over cap | APO update runs | capped sample set is curriculum-weighted and report focus category is weakest |
| C-03 | AC-3 | Regression/Conformance | category with enough historical samples and changing rewards | new span is persisted | `learning_trend` reflects deterministic trend state |
| C-04 | AC-4 | Functional/Conformance | Review #37 doc | doc validation | OTel status marked implemented with references |

## Success Metrics / Observable Signals
- `cargo test -p tau-coding-agent --bin tau-coding-agent live_rl_runtime::tests::spec_c12_functional_live_span_persists_meta_cognition_fields`
- `cargo test -p tau-coding-agent --bin tau-coding-agent live_rl_runtime::tests::spec_c13_regression_live_apo_curriculum_prioritizes_weak_categories`
- `cargo test -p tau-coding-agent --bin tau-coding-agent live_rl_runtime::tests::spec_c14_regression_live_span_learning_trend_regressing_when_recent_reward_drops`
- `cargo test -p tau-coding-agent --bin tau-coding-agent live_rl_runtime::tests::spec_c02_functional_optimizer_runs_on_update_interval`
- `cargo fmt --check`
- `cargo clippy -p tau-coding-agent --no-deps -- -D warnings`
