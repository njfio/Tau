# Issue 1756 Tasks

Status: Implementing

## Ordered Tasks

T1 (tests-first): add failing tests for legacy alias compatibility parsing and
warning text snapshot assertions.

T2: implement legacy alias normalization + warning generation in `tau-cli`.

T3: wire startup/test parse flows in `tau-coding-agent` through the normalizer.

T4: run targeted unit/functional/regression tests and capture RED/GREEN evidence.

## Tier Mapping

- Unit: alias normalization mapping + warning string assertions
- Functional: legacy flags parse into canonical fields
- Regression: `--legacy=value` path and unknown flag fail-closed behavior
