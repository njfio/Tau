# M293 - Operator integration closure wave

Status: InProgress

## Context
M295/M296 readiness gates validate deterministic baseline delivery, but remaining integration work is still required to make the operator experience fully coherent across documentation contracts, live terminal operations, dashboard control mutations, RL hardening checks, and live-environment auth validation coverage.

## Scope
- Execute issue `#3444` as the implementation task for this milestone.
- Close the README docs-contract blocker required by capability/archive conformance checks.
- Add a state-backed live TUI shell mode (beyond deterministic fixture-only shell rendering).
- Wire dashboard control actions into a live operator route flow.
- Expand RL and live auth validation checks in readiness verification scripts/runbooks.

## Linked Issues
- Epic: #3442
- Story: #3443
- Task: #3444

## Success Signals
- `scripts/dev/test-docs-capability-archive.sh` passes.
- New live TUI mode renders runtime state data from dashboard artifacts.
- Dashboard ops shell exposes an action submit path that mutates control state via gateway action runtime.
- M296 gate includes expanded RL/auth validation checks and remains green.
- `specs/3444/spec.md`, `specs/3444/plan.md`, and `specs/3444/tasks.md` are present and advanced to Implemented on close.
