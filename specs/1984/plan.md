# Issue 1984 Plan

Status: Reviewed

## Approach

1. Add report model(s) in `benchmark_artifact.rs`:
   - `BenchmarkArtifactGateSummaryReport`
2. Add builder:
   - `build_benchmark_artifact_gate_summary_report(summary, policy)`
   - evaluate quality via existing
     `evaluate_benchmark_gate_report_summary_quality(...)`
   - return combined report
3. Add `to_json_value()` with nested `summary` and `quality` sections.
4. Add tests for C-01..C-04 and a zero-summary regression guard.

## Affected Areas

- `crates/tau-trainer/src/benchmark_artifact.rs`
- `specs/1984/spec.md`
- `specs/1984/plan.md`
- `specs/1984/tasks.md`

## Risks And Mitigations

- Risk: drift between summary counters and quality-input mapping.
  - Mitigation: quality evaluation uses the summary object directly.
- Risk: unstable payload shape.
  - Mitigation: fixed top-level keys with explicit nested sections.

## ADR

No dependency/protocol changes; ADR not required.
