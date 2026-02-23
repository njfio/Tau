# Tasks: Issue #3400 - F10 fallback/circuit-breaker scenarios

- [x] T1 (RED): add spec-derived failing tests for `F10-01` primary success/no-fallback and `F10-03` all-routes-fail behavior.
- [x] T2 (GREEN): implement minimal fallback-router/test harness adjustments so new tests pass deterministically.
- [x] T3 (TRACE): map executable tests to `F10-01/02/03/04/05/08` conformance rows.
- [x] T4 (VERIFY): run fmt/clippy/targeted tests/crate tests/mutants and record results in issue + PR.
