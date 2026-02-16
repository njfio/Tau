# Issue 1978 Plan

Status: Reviewed

## Approach

1. Add gate report export helper in `benchmark_artifact.rs`:
   - `export_benchmark_artifact_gate_report(report, output_dir)`
   - deterministic filename derived from report counters
2. Add replay validator helper:
   - `validate_exported_benchmark_artifact_gate_report(path)`
   - enforce top-level object with required `manifest` and `quality` keys
3. Add tests for C-01..C-04 and one malformed-payload regression guard.

## Affected Areas

- `crates/tau-trainer/src/benchmark_artifact.rs`
- `specs/1978/spec.md`
- `specs/1978/plan.md`
- `specs/1978/tasks.md`

## Risks And Mitigations

- Risk: payload drift between in-memory report and exported JSON.
  - Mitigation: export always serializes `report.to_json_value()`.
- Risk: non-deterministic export names breaking automation references.
  - Mitigation: deterministic filename from stable report counters.

## ADR

No dependency/protocol changes; ADR not required.
