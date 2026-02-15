# Issue 1710 Tasks

Status: Reviewed

## Ordered Tasks

T1 (tests-first): add failing script tests for proof artifact pass/fail behavior.

T2: implement `m24-rl-operational-safety-proof.sh` with deterministic check
execution and artifact emission.

T3: add runbook evidence path check and include metadata in JSON/MD artifacts.

T4: update docs with operator invocation path for proof generation.

T5: run scoped verification and map AC-1..AC-4 to C-01..C-04.

## Tier Mapping

- Unit: script argument and status mapping assertions
- Functional: pass-runner proof artifact generation
- Integration: artifact schema + command list + runbook evidence fields
- Regression: fail-runner non-zero exit and failing check capture
- Conformance: C-01..C-04
