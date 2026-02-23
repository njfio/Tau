# Tasks: Issue #3430 - M296 GA readiness gate, docs, and rollback validation

1. [x] T1 (RED, Conformance): add failing test coverage in `scripts/verify/test-m296-ga-readiness-gate.sh` for required report schema fields and fail-closed exit behavior.
2. [x] T2 (GREEN, Verification): implement `scripts/verify/m296-ga-readiness-gate.sh` with deterministic step logging, signoff criteria, rollback trigger matrix, and milestone closeout summary.
3. [x] T3 (GREEN, Docs): add/update operator-facing docs so README/docs index expose one connected GA command flow and runbook linkage.
4. [x] T4 (Regression): execute new script tests and existing readiness/rollback tests to confirm no regressions.
5. [x] T5 (Verify): run scoped formatting/lint checks for touched shell/docs/spec artifacts and update spec status to `Implemented`.

## RED / GREEN Evidence

### RED
- `bash scripts/verify/test-m296-ga-readiness-gate.sh`
- Expected failure before implementation:
  - `No such file or directory: scripts/verify/m296-ga-readiness-gate.sh`

### GREEN
- `bash scripts/verify/test-m296-ga-readiness-gate.sh`
- `bash scripts/verify/m296-ga-readiness-gate.sh`

### Regression
- `bash scripts/dev/test-operator-readiness-live-check.sh`
- `bash scripts/demo/test-rollback-drill-checklist.sh`
- `python3 .github/scripts/runbook_ownership_docs_check.py`
- `cargo fmt --all -- --check`
