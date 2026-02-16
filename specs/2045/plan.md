# Plan #2045

Status: Implemented
Spec: specs/2045/spec.md

## Approach

1. Re-run the merged M25.4.1a baseline generator in live mode to refresh
   task-level baseline artifacts with real command timings.
2. Verify shell functional/regression and Python contract suites.
3. Record task-level conformance mapping and closure evidence referencing `#2068`
   implementation assets.

## Affected Modules

- `tasks/reports/m25-build-test-latency-baseline.json`
- `tasks/reports/m25-build-test-latency-baseline.md`
- `scripts/dev/build-test-latency-baseline.sh` (verification target)
- `scripts/dev/test-build-test-latency-baseline.sh` (verification target)
- `.github/scripts/test_build_test_latency_baseline_contract.py`
- `specs/2045/spec.md`
- `specs/2045/plan.md`
- `specs/2045/tasks.md`

## Risks and Mitigations

- Risk: live timing numbers drift between runs.
  - Mitigation: keep fixed command set and document generated timestamp +
    environment metadata.
- Risk: baseline report no longer reflects conformance constraints.
  - Mitigation: shell + contract tests are required before closure.

## Interfaces and Contracts

- Generator:
  `scripts/dev/build-test-latency-baseline.sh`
- Functional/regression suite:
  `scripts/dev/test-build-test-latency-baseline.sh`
- Contract suite:
  `python3 .github/scripts/test_build_test_latency_baseline_contract.py`

## ADR References

- Not required.
