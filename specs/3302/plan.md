# Plan: Issue #3302

## Approach
1. Add RED tests for curriculum aggregate persistence, progressive difficulty selection, calibration curve/alerts, and gateway alert ingestion.
2. Extend live RL runtime with:
   - canonical taxonomy normalization
   - long-horizon curriculum summary builder
   - calibration-curve + alert synthesis
   - merged resource/status persistence of curriculum/meta-cognition summaries.
3. Update APO sample capping to consume difficulty weights from latest summaries for progressive scheduling.
4. Extend gateway dashboard status loading to ingest live learning alerts from training status artifacts and append them to dashboard alerts.
5. Re-run targeted tests and scoped lint/format gates.

## Affected Modules
- `crates/tau-coding-agent/src/live_rl_runtime.rs`
- `crates/tau-gateway/src/gateway_openresponses/dashboard_status.rs`
- `tasks/review-37.md`
- `specs/milestones/m251/index.md`

## Risks and Mitigations
- Risk: Additional status/resource writes may overwrite prompt resources.
  Mitigation: merged-write helper that preserves latest resource keys while patching insights.
- Risk: Difficulty weighting could over-focus a single category.
  Mitigation: bounded progressive allocation with fallback round-robin and deterministic caps.
- Risk: Dashboard alerts could become noisy.
  Mitigation: deterministic thresholds, capped alert list, and severity mapping.

## Interfaces / Contracts
- Internal status/resource contract additions:
  - `live_curriculum_*` keys
  - `live_meta_cognition_*` keys
  - `live_learning_alerts` list in training status payload.
- No public wire-format break; added fields are backward-compatible optional fields.

## ADR
Not required.
