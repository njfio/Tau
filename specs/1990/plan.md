# Issue 1990 Plan

Status: Reviewed

## Approach

1. Add quality models in `benchmark_artifact.rs`:
   - `BenchmarkArtifactGateSummaryReportManifestQualityPolicy`
   - `BenchmarkArtifactGateSummaryReportManifestQualityDecision`
2. Add evaluator helper:
   - `evaluate_benchmark_gate_summary_report_manifest_quality(manifest, policy)`
   - derive totals and ratios from manifest counters
   - emit deterministic reason codes
3. Add `to_json_value()` for quality decision.
4. Add tests for C-01..C-04 plus zero-entry regression guard.

## Affected Areas

- `crates/tau-trainer/src/benchmark_artifact.rs`
- `specs/1990/spec.md`
- `specs/1990/plan.md`
- `specs/1990/tasks.md`

## Risks And Mitigations

- Risk: ratio math drift for zero-entry manifests.
  - Mitigation: explicit zero-denominator handling (`0.0`) and regression test.
- Risk: ambiguous reason ordering.
  - Mitigation: deterministic append order for reason codes.

## ADR

No dependency/protocol changes; ADR not required.
