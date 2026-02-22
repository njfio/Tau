# Plan: Issue #3300

## Approach
1. Add RED tests for meta-cognition span fields, curriculum sample prioritization, and trend classification.
2. Extend live span assembly with deterministic category inference and confidence outcome signals.
3. Add history-based meta-cognition enrichment that computes calibration error and ask-for-help recommendation from recent category outcomes.
4. Replace APO sample cap behavior with curriculum-weighted selection and expose focus category in APO report.
5. Update Review #37 markdown to remove stale OpenTelemetry "not started" claim and align missing-work status.
6. Re-run targeted tests, then crate lint/format gates.

## Affected Modules
- `crates/tau-coding-agent/src/live_rl_runtime.rs`
- `tasks/review-37.md`
- `specs/milestones/m251/index.md`

## Risks and Mitigations
- Risk: new metadata logic can make optimizer updates brittle if span history parsing fails.
  Mitigation: fail-open enrichment (span persists even if enrichment lookup fails) and deterministic defaults.
- Risk: curriculum weighting could starve high-performing categories.
  Mitigation: bounded round-robin selection across all categories after weakness ordering.
- Risk: trend classification sensitivity could flap.
  Mitigation: fixed window sizes and explicit thresholds with regression tests.

## Interfaces / Contracts
- Internal-only `LiveApoReport` contract extension for curriculum focus diagnostics.
- Internal-only live span attributes extension for meta-cognition telemetry.
- No wire-format/API contract changes.

## ADR
Not required.
