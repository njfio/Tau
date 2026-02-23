# M296 GA Readiness Gate

Run all commands from repository root.

## Purpose

This runbook is the final M296 go/no-go gate for integrated operator delivery across:
- true RL verification,
- auth and readiness workflow validation,
- rollback trigger contract checks,
- milestone closeout evidence packaging.

Canonical command:

```bash
./scripts/verify/m296-ga-readiness-gate.sh
```

Output artifact:

- `artifacts/operator-ga-readiness/verification-report.json`

## What The Gate Executes

The gate runs these checks in order:

1. `scripts/verify/m295-operator-maturity-wave.sh`
2. `scripts/dev/test-operator-readiness-live-check.sh`
3. `scripts/demo/test-rollback-drill-checklist.sh`
4. rollback-trigger marker checks in `docs/guides/consolidated-runtime-rollback-drill.md`
5. README GA workflow marker checks
6. docs index GA entry checks
7. milestone/spec artifact presence checks

Any failed step sets `overall=fail` and exits non-zero.

## Signoff Criteria

The report emits four explicit signoff criteria:

1. `runtime_and_auth_contracts`
2. `rollback_contract`
3. `docs_connected_flow`
4. `milestone_closeout_artifacts`

Promotion requires all four criteria to be `pass` and `closeout_summary.status=ready`.

## Rollback Trigger Matrix Source

Rollback trigger contract source:

- `docs/guides/consolidated-runtime-rollback-drill.md`

Trigger IDs tracked in the GA report:

- `proof-summary-missing`
- `proof-runs-failed`
- `proof-markers-missing`
- `validation-matrix-missing`
- `validation-open-issues`
- `validation-completion-below-100`

## Evidence Collection

Store and attach at minimum:

- `artifacts/operator-ga-readiness/verification-report.json`
- `artifacts/operator-maturity-wave/verification-report.json`
- logs referenced in `steps[].log` from the GA report

## Related Runbooks

- `docs/guides/ops-readiness-live-validation.md`
- `docs/guides/consolidated-runtime-rollback-drill.md`
- `docs/guides/operator-deployment-guide.md`
