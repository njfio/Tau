# Issue 1672 Plan

Status: Reviewed

## Approach

1. Add `scripts/demo/m24-rl-safety-regression-benchmark.sh` to:
   - read baseline/trained safety sample vectors
   - compute means and regression delta
   - compare delta against threshold
   - emit benchmark report with deterministic reason codes
2. Add `scripts/demo/test-m24-rl-safety-regression-benchmark.sh` for pass/fail
   cases and report-field assertions.
3. Integrate safety benchmark invocation into
   `scripts/demo/m24-rl-live-benchmark-proof.sh` and include artifact path +
   decision details in proof output.
4. Update docs with safety benchmark command and integration notes.

## Affected Areas

- `scripts/demo/m24-rl-safety-regression-benchmark.sh` (new)
- `scripts/demo/test-m24-rl-safety-regression-benchmark.sh` (new)
- `scripts/demo/m24-rl-live-benchmark-proof.sh`
- `scripts/demo/test-m24-rl-live-benchmark-proof.sh`
- `docs/guides/training-ops.md`
- `docs/README.md`
- `specs/1672/spec.md`
- `specs/1672/plan.md`
- `specs/1672/tasks.md`

## Risks And Mitigations

- Risk: threshold interpretation mismatch across scripts.
  - Mitigation: centralize threshold fields and reason codes in safety benchmark
    report; consume these in proof flow.
- Risk: silent pass on malformed safety vectors.
  - Mitigation: strict finite-number validation with fail-closed exits.
- Risk: proof-flow compatibility regressions.
  - Mitigation: run existing live-proof harness after integration.

## ADR

No architecture/dependency/protocol change; ADR not required.
