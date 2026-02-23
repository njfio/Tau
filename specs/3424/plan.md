# Plan: Issue #3424 - Full program delivery execution plan

## Approach
Build and execute the full-delivery program in six phases, each with explicit gates and child-task decomposition:

1. Baseline and contract freeze
2. True RL end-to-end productionization
3. Dashboard and TUI convergence
4. Auth lifecycle hardening and validation
5. Integrated end-to-end/system reliability proof
6. GA cutover with rollback posture

This issue delivers the plan artifact set and execution contract. Runtime implementation work is split into child tasks under milestone `M296`.

## Phase Plan

### Phase 0 - Baseline Freeze and Gap Snapshot
Outputs:
- Runtime baseline snapshot (RL/dashboard/TUI/auth surfaces).
- Existing test/coverage inventory per stream.
- Open-gap list with priority and dependency tags.

Gate:
- Baseline evidence committed and linked by child task.

### Phase 1 - True RL End-to-End Productionization
Outputs:
- RL runtime contract and acceptance matrix:
  - rollout capture and persistence,
  - reward inference determinism and observability,
  - optimizer cadence and safety guardrails,
  - statistically significant improvement gate,
  - rollback path when regressions are detected.
- RL dashboards/TUI signals required for operators.

Gate:
- RL conformance tests and targeted regression suite pass.

### Phase 2 - Dashboard and TUI Convergence
Outputs:
- Shared operator workflow map (same intents in both surfaces).
- Data contract parity checklist for live status, sessions, jobs, memory, safety, and RL views.
- UX acceptance matrix for common and failure workflows.

Gate:
- Cross-surface parity tests pass and no blocked operator-critical flows remain.

### Phase 3 - Auth Workflow Validation and Hardening
Outputs:
- Auth lifecycle matrix covering:
  - bootstrap/initial auth mode resolution,
  - token and password session workflows,
  - session expiry and logout,
  - credential/key rotation,
  - revocation/invalid token handling,
  - fail-closed negative paths.
- Deterministic integration/regression tests for each path.

Gate:
- Auth matrix is fully covered (no untracked paths) with passing tests.

### Phase 4 - Integrated E2E Reliability and Recovery
Outputs:
- End-to-end integrated flows that include RL + dashboard/TUI + auth in one path.
- Chaos/recovery checks for disconnects, restart continuity, and degraded dependencies.
- Operational observability checks (logs/events/alerts aligned to operator actions).

Gate:
- Integrated reliability suite passes and recovery behavior is verified.

### Phase 5 - GA Readiness and Rollback
Outputs:
- GA readiness checklist:
  - AC coverage complete,
  - test tier matrix complete (or justified N/A),
  - docs/runbooks updated,
  - migration/release notes prepared.
- Rollback plan with concrete triggers and operator actions.

Gate:
- Release checklist signed off in PR evidence and milestone closeout.

## Issue Decomposition Strategy
- Create child tasks per phase stream, each with dedicated `spec.md`, `plan.md`, and `tasks.md`.
- Enforce one-child-one-scope to keep diffs and verification focused.
- Require each child task to include AC -> conformance -> tests mapping before merge.

## Affected Modules
- `specs/milestones/m296/index.md`
- `specs/3424/spec.md`
- `specs/3424/plan.md`
- `specs/3424/tasks.md`
- Child tasks (to be created) for runtime/docs/test implementation under milestone `M296`.

## Risks and Mitigations
- Risk: "true RL" claims drift from implemented behavior.
  - Mitigation: require explicit acceptance gates tied to deterministic tests and runtime evidence.
- Risk: dashboard and TUI evolve independently and diverge.
  - Mitigation: parity matrix and shared workflow contracts as merge gates.
- Risk: auth coverage misses edge-case failure modes.
  - Mitigation: lifecycle matrix includes negative-path regressions as required cases.
- Risk: integration regressions only discovered late.
  - Mitigation: phase 4 integrated suites run before GA gate.

## Interfaces / Contracts
- Gateway auth/session/runtime contracts.
- RL/training runtime contracts and telemetry signals.
- Dashboard and TUI operator contract parity for shared workflows.
- Integration test selectors for PR/nightly/weekly gates.

## ADR
No ADR required for this planning issue itself.

Child tasks must add ADRs if they introduce new dependencies, protocol changes, or major architecture changes.
