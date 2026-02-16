# Issue 1996 Plan

Status: Reviewed

## Approach

1. Add typed M24 evidence bundle structs in `benchmark_artifact.rs`:
   - `M24RLGateEvidenceBundle`
   - `M24RLGateEvidenceBundleInput`
   - nested benchmark/safety/operations/runbook section structs
2. Add deterministic builder:
   - `build_m24_rl_gate_evidence_bundle(input)`
   - preserve deterministic nested sections and pass/fail signals
3. Add deterministic export and replay validator:
   - `export_m24_rl_gate_evidence_bundle(bundle, output_dir)`
   - `validate_exported_m24_rl_gate_evidence_bundle(path)`
4. Add C-01..C-04 tests plus missing-section regression guard.

## Affected Areas

- `crates/tau-trainer/src/benchmark_artifact.rs`
- `specs/1996/spec.md`
- `specs/1996/plan.md`
- `specs/1996/tasks.md`

## Risks And Mitigations

- Risk: section drift between in-memory bundle and exported JSON.
  - Mitigation: export serializes `bundle.to_json_value()` directly.
- Risk: non-deterministic output naming.
  - Mitigation: deterministic filename derives from schema + pass/fail signals.

## ADR

No dependency/protocol changes; ADR not required.
