# Spec: Issue #3430 - M296 GA readiness gate, docs, and rollback validation

Status: Implemented

## Problem Statement
M296 needs one final, deterministic GA gate that unifies readiness evidence, rollback trigger validation, and operator-facing closeout docs. Current validation assets exist across multiple scripts/runbooks but are not yet emitted as one explicit milestone closeout report with signoff criteria.

## Scope
In scope:
- Add a root verification script for M296 GA readiness that emits a deterministic report artifact.
- Define explicit GA signoff criteria and include pass/fail status in report output.
- Validate rollback trigger matrix coverage against the canonical rollback runbook contract.
- Update operator-facing docs (README + docs index/runbook) so the GA flow is discoverable as one connected workflow.
- Add script-level tests for pass and fail-closed behaviors.

Out of scope:
- New runtime/auth protocol behavior changes.
- External production credential/network validation.
- Replacing prior phase verification suites.

## Acceptance Criteria
### AC-1 Deterministic GA readiness report exists
Given repository-root execution of the M296 GA verification command,  
when the script runs,  
then it emits `artifacts/operator-ga-readiness/verification-report.json` with explicit signoff criteria, step results, and closeout summary fields.

### AC-2 Rollback trigger contract is validated fail-closed
Given rollback drill and runbook artifacts,  
when rollback checks fail or required trigger/runbook markers are missing,  
then the GA verification result is `overall=fail` and exits non-zero.

### AC-3 Operator docs present one connected GA path
Given an operator using root documentation entrypoints,  
when they follow GA guidance,  
then they can discover one connected command flow for RL, auth, readiness, rollback, and final signoff.

### AC-4 Test-first coverage demonstrates red/green behavior
Given the new GA verification script contract,  
when test harnesses run,  
then they prove both pass-path and fail-closed behavior for report generation and exit codes.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Conformance | M296 verification command | run `scripts/verify/m296-ga-readiness-gate.sh` | JSON report exists with `suite_id`, `overall`, `signoff_criteria[]`, `steps[]`, `closeout_summary` |
| C-02 | AC-1 | Functional | all scripted checks pass | run M296 gate | `overall=pass`, all signoff criteria pass, exit code 0 |
| C-03 | AC-2 | Regression | one required check fails | run M296 gate with fail injection | `overall=fail`, failing step recorded, exit code non-zero |
| C-04 | AC-2 | Functional | rollback runbook contract | run M296 gate | report includes rollback trigger matrix source + trigger IDs and validates docs marker presence |
| C-05 | AC-3 | Functional | README/docs index/runbook | inspect docs entrypoints | GA readiness gate command and connected workflow links are present |
| C-06 | AC-4 | Regression | script test harness | run `scripts/verify/test-m296-ga-readiness-gate.sh` | pass/fail paths are validated deterministically |

## Success Metrics / Observable Signals
- Operators can run one command to generate a final GA gate report with explicit signoff fields.
- Rollback trigger contract is validated by executable checks, not narrative only.
- README and docs index present a connected integrated path rather than isolated components.
- Script-level tests provide deterministic RED/GREEN evidence.

## Implementation Evidence
### C-01/C-02 (deterministic GA gate report + pass path)
- `bash scripts/verify/m296-ga-readiness-gate.sh`
- Artifact: `artifacts/operator-ga-readiness/verification-report.json`

### C-03/C-06 (fail-closed behavior + test harness)
- `bash scripts/verify/test-m296-ga-readiness-gate.sh`
- Key assertions:
  - pass path: `overall=pass`, `closeout_summary.status=ready`
  - fail path: `overall=fail`, `rollback_contract` criterion fails, exit code non-zero

### C-04 (rollback trigger matrix validation)
- `bash scripts/demo/test-rollback-drill-checklist.sh`
- `rg -n "proof-summary-missing|proof-runs-failed|proof-markers-missing|validation-matrix-missing|validation-open-issues|validation-completion-below-100" docs/guides/consolidated-runtime-rollback-drill.md`

### C-05 (docs connected workflow)
- `rg -n "M296 GA readiness gate|scripts/verify/m296-ga-readiness-gate.sh|Connected operator GA loop" README.md`
- `rg -n "M296 GA Readiness Gate|guides/m296-ga-readiness-gate.md" docs/README.md`
