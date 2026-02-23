# M296 - Full Program Delivery (RL + Dashboard + Auth + TUI GA)

Status: In Progress

## Context
Tau has broad runtime capabilities, but program-level delivery needs explicit integration across:
- true RL production behavior and operator controls,
- dashboard and TUI as one coherent operations plane,
- complete auth lifecycle verification,
- release/readiness gates that prove the system behaves as one program.

Milestone `M296` is the integration and productionization program for that outcome.

## Scope
- Define and execute a phased program for RL, dashboard, auth, and TUI convergence.
- Establish binding acceptance gates and conformance evidence for each phase.
- Decompose execution into task issues with explicit dependencies and rollback posture.
- Validate end-to-end operator workflows, not only component-level behavior.

## Linked Issues
- Epic: #3422
- Story: #3423
- Task: #3424

## Exit Criteria
- Program decomposition is complete and executable (phase plan + child tasks + dependencies).
- True RL path is defined with measurable runtime and verification gates.
- Dashboard and TUI convergence plan is defined with parity and integration criteria.
- Auth workflows (token/password/bootstrap/rotation/expiry/logout) have explicit validation matrix.
- Release gate checklist and rollback strategy are documented and test-backed.

## Success Signals
- `specs/3424/spec.md`, `specs/3424/plan.md`, `specs/3424/tasks.md` exist and are maintained.
- Each child implementation issue under this milestone maps AC -> conformance -> tests.
- Milestone closeout records passed gates for RL, dashboard/TUI, auth, and release readiness.
