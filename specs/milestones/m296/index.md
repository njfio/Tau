# M296 - Full Program Delivery (RL + Dashboard + Auth + TUI GA)

Status: In Progress

## Context
M296 is the integrated program milestone for delivering production-grade operator flows across:
- true RL end-to-end runtime evidence,
- dashboard and TUI operator-surface convergence,
- auth lifecycle verification and hardening,
- GA readiness gates with rollback and documentation contracts.

## Scope
- Execute story `#3423` and its child task phases.
- Ensure every phase ships with spec/plan/tasks artifacts and deterministic verification evidence.
- Publish a final GA closeout path that operators can run from repository root.

## Linked Issues
- Epic: `#3422`
- Story: `#3423`
- Task: `#3424` (execution plan authoring)
- Task: `#3427` (true RL e2e productionization gates)
- Task: `#3428` (dashboard + TUI convergence)
- Task: `#3426` (auth lifecycle verification)
- Task: `#3429` (integrated reliability + recovery proof)
- Task: `#3430` (GA readiness gate, docs, rollback validation)

## Phase Outcomes
| Phase | Issue | Goal |
| --- | --- | --- |
| 1 | `#3427` | RL end-to-end productionization gates |
| 2 | `#3428` | Dashboard/TUI operator-flow parity contracts |
| 3 | `#3426` | Full auth workflow conformance and hardening |
| 4 | `#3429` | Integrated reliability and recovery verification |
| 5 | `#3430` | Final GA gate, rollback criteria, docs, closeout summary |

## GA Exit Signals
- M295 operator maturity wave verification passes:
  - `scripts/verify/m295-operator-maturity-wave.sh`
- M296 GA readiness gate verification passes:
  - `scripts/verify/m296-ga-readiness-gate.sh`
- Final GA report is generated and marked pass:
  - `artifacts/operator-ga-readiness/verification-report.json`

## Verification Artifacts
- `scripts/verify/m295-operator-maturity-wave.sh`
- `scripts/verify/m296-ga-readiness-gate.sh`
- `artifacts/operator-maturity-wave/verification-report.json`
- `artifacts/operator-ga-readiness/verification-report.json`
