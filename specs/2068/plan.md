# Plan #2068

Status: Reviewed
Spec: specs/2068/spec.md

## Approach

1. Add a deterministic script `scripts/dev/build-test-latency-baseline.sh`
   that supports:
   - fixture mode for contract tests/reproducibility,
   - live mode for executing command rows and measuring elapsed milliseconds.
2. Emit canonical artifacts:
   - `tasks/reports/m25-build-test-latency-baseline.json`
   - `tasks/reports/m25-build-test-latency-baseline.md`
3. Add schema + tests:
   - shell functional/regression test script,
   - Python contract test for schema + doc references.
4. Document operator workflow in `docs/guides/build-test-latency-baseline.md`.

## Affected Modules

- `scripts/dev/build-test-latency-baseline.sh`
- `scripts/dev/test-build-test-latency-baseline.sh`
- `.github/scripts/test_build_test_latency_baseline_contract.py`
- `tasks/schemas/m25-build-test-latency-baseline.schema.json`
- `tasks/reports/m25-build-test-latency-baseline.json`
- `tasks/reports/m25-build-test-latency-baseline.md`
- `docs/guides/build-test-latency-baseline.md`
- `specs/2068/{spec.md,plan.md,tasks.md}`

## Risks and Mitigations

- Risk: live command runs are long/flaky and slow local iteration.
  - Mitigation: fixture mode is first-class; live mode is opt-in and scoped via
    explicit command list and iteration count.
- Risk: environment metadata drift makes comparisons unreliable.
  - Mitigation: artifact requires source mode + toolchain + platform metadata.
- Risk: output format drifts and breaks downstream budget work.
  - Mitigation: schema + contract tests enforce stable fields and ordering.

## Interfaces and Contracts

- Generator:
  `scripts/dev/build-test-latency-baseline.sh --output-json <path> --output-md <path>`
- Fixture mode:
  `--fixture-json <path> --generated-at <iso>`
- Live mode:
  `--command \"id::command\"` (repeatable) `--iterations <n>`
- Schema:
  `tasks/schemas/m25-build-test-latency-baseline.schema.json`
- Validation:
  `scripts/dev/test-build-test-latency-baseline.sh`
  `python3 .github/scripts/test_build_test_latency_baseline_contract.py`

## ADR References

- Not required.
