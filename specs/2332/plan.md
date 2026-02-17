# Plan #2332

Status: Reviewed
Spec: specs/2332/spec.md

## Approach

1. Capture RED evidence showing wave-2 verifier does not yet call
   `verify-session-postgres-live.sh`.
2. Update claim #6 mapping in `scripts/dev/verify-gap-claims-wave2.sh` to call
   the live verifier script.
3. Run the updated wave-2 verifier end-to-end and capture GREEN evidence.

## Affected Modules

- `scripts/dev/verify-gap-claims-wave2.sh`
- `specs/milestones/m53/index.md`
- `specs/2332/spec.md`
- `specs/2332/plan.md`
- `specs/2332/tasks.md`

## Risks and Mitigations

- Risk: wave-2 run time increases due live Postgres spin-up.
  - Mitigation: reuse existing fast target dir and keep single delegated call.

## Interfaces / Contracts

- Delegation contract: wave-2 verifier calls
  `scripts/dev/verify-session-postgres-live.sh` for claim #6.
- Fail-closed contract: delegated command failure propagates non-zero exit.
