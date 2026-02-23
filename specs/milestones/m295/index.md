# M295 - Operator maturity wave (real TUI + RL e2e + auth verification)

Status: Implemented

## Context
Tau has strong core runtime capabilities, but operator confidence depends on three connected upgrades:
- a practical terminal operator experience (`tau-tui`),
- an executable RL end-to-end flow path (not only primitives),
- comprehensive auth workflow validation across provider and gateway surfaces.

## Scope
- Deliver issue `#3420` under story `#3418` and epic `#3419`.
- Ship code + tests for TUI, RL e2e harnessing, and auth conformance coverage.
- Provide deterministic verification evidence and docs/spec status updates.

## Linked Issues
- Epic: #3419
- Story: #3418
- Task: #3420

## Success Signals
- `tau-tui` has a real operator-shell mode with structured panel output and tests.
- RL e2e harness runs and emits a deterministic artifact from integrated training components.
- Auth workflow coverage includes provider-mode matrix and gateway auth/session validation tests.

## Verification Artifacts
- Root verification script: `scripts/verify/m295-operator-maturity-wave.sh`
- Verification report: `artifacts/operator-maturity-wave/verification-report.json`
