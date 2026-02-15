# Issue 1740 Tasks

Status: Implementing

## Ordered Tasks

T1 (tests-first): add failing integration/regression tests for crash-recovery
runtime behavior and playbook artifact validation.

T2: implement runtime crash-restart recovery drill coverage in
`background_jobs_runtime` tests.

T3: add playbook artifact template + validator + validator tests.

T4: document drill steps and evidence paths in training operations guide.

T5: run fmt/clippy/tests and verify AC mapping.

## Tier Mapping

- Integration: restart recovery drill in runtime
- Functional: valid playbook artifact acceptance
- Regression: malformed playbook artifact rejection
