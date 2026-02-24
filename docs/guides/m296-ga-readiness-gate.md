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
2. `scripts/demo/test-m24-rl-live-benchmark-proof.sh`
3. `scripts/dev/test-operator-readiness-live-check.sh`
4. `scripts/verify/m296-live-auth-validation.sh`
5. `scripts/demo/test-rollback-drill-checklist.sh`
6. rollback-trigger marker checks in `docs/guides/consolidated-runtime-rollback-drill.md`
7. README GA workflow marker checks
8. docs index GA entry checks
9. milestone/spec artifact presence checks

Any failed step sets `overall=fail` and exits non-zero.

Live auth validation step notes:
- The step exits `skip` when `TAU_M296_AUTH_LIVE_ENABLE` is not set to `1`, provider key file is absent, or no usable provider keys are present.
- Configure live auth validation with:
  - `TAU_M296_AUTH_LIVE_ENABLE=1`
  - `TAU_PROVIDER_KEYS_FILE=/absolute/path/to/provider-keys.env`

## Signoff Criteria

The report emits six explicit signoff criteria:

1. `runtime_and_auth_contracts`
2. `rl_hardening_contracts`
3. `auth_live_validation`
4. `rollback_contract`
5. `docs_connected_flow`
6. `milestone_closeout_artifacts`

Promotion requires all six criteria to be `pass` and `closeout_summary.status=ready`.
`auth_live_validation` is counted as `pass` when the step is `pass` or `skip` (skip is only valid for missing live-env inputs).

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
