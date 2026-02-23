# Tasks: Issue #3429 - Integrated reliability and recovery proof

- [x] T1 (RED, Functional/Regression/Conformance): add failing harness for integrated reliability suite pass/fail/report expectations.
- [x] T2 (GREEN, Implementation): implement `m296-integrated-reliability-wave` suite with deterministic report and fail-closed behavior.
- [x] T3 (REGRESSION, Functional): run harness and adjacent verification checks.
- [x] T4 (VERIFY, Conformance): capture AC -> test evidence and mark spec status implemented.

## AC -> Conformance -> Tests
| AC | Conformance Case(s) | Test(s) |
|---|---|---|
| AC-1 | C-01 | `scripts/verify/test-m296-integrated-reliability-wave.sh` (asserts required step ids and count) |
| AC-2 | C-02 | `scripts/verify/test-m296-integrated-reliability-wave.sh` (asserts deterministic report schema/suite/overall/steps) |
| AC-3 | C-03 | `scripts/verify/test-m296-integrated-reliability-wave.sh` (mock fail pattern -> non-zero exit + failed step in report) |
| AC-4 | C-04 | `specs/3429/{spec.md,plan.md,tasks.md}` |

## TDD Evidence
### RED
- `bash scripts/verify/test-m296-integrated-reliability-wave.sh`
- Result: failed with `verification script missing or not executable` because `m296-integrated-reliability-wave.sh` did not exist yet.

### GREEN
- `bash scripts/verify/test-m296-integrated-reliability-wave.sh`
- Result: pass; harness validates pass path + fail-closed path and report shape for `m296_integrated_reliability_wave`.

### REGRESSION / VERIFY
- `bash scripts/verify/test-m296-integrated-reliability-wave.sh` -> pass.
- `bash scripts/dev/test-operator-readiness-live-check.sh` -> pass.

## Tier Mapping
- Unit: N/A (shell suite/harness behavior scope).
- Property: N/A (no randomized invariant changes).
- Contract/DbC: N/A (no DbC annotation changes).
- Snapshot: N/A.
- Functional: verification suite step execution and report generation.
- Conformance: AC/C-case mapping via harness assertions and report checks.
- Integration: cross-stream command matrix (restart/reconnect/degraded/auth/RL).
- Fuzz: N/A (no new parser surface).
- Mutation: N/A (verification workflow hardening slice).
- Regression: fail-closed suite behavior when one step fails.
- Performance: N/A (no runtime performance path changes).
