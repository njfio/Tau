# Spec: Issue #3424 - Full-delivery execution plan and acceptance program

Status: Reviewed

## Problem Statement
Current capabilities exist across RL, dashboard, auth, and TUI surfaces, but program-level delivery requires a single executable plan with clear phase ordering, acceptance gates, and verification strategy. Without that plan, execution risks fragmentation and non-integrated outcomes.

## Scope
In scope:
- Define an executable phased delivery plan for:
  - true RL end-to-end production behavior,
  - dashboard and TUI convergence,
  - full auth workflow validation,
  - release and rollback readiness.
- Define acceptance criteria, conformance cases, and test-tier expectations for each stream.
- Define issue decomposition and dependency sequencing under milestone `M296`.

Out of scope:
- Implementing runtime feature changes in this issue.
- Changing production APIs/wire formats in this issue.
- Completing deployment cutover in this issue.

## Acceptance Criteria
### AC-1 Program decomposition is executable
Given milestone `M296`,
when `#3424` artifacts are reviewed,
then an ordered phase plan, dependency map, and child-task decomposition are defined with no unresolved ownership gaps.

### AC-2 True RL production path is explicitly defined
Given current RL/training surfaces,
when the plan is inspected,
then it includes concrete deliverables and gates for rollout capture, reward inference, optimizer cadence, significance gating, persistence, observability, and rollback controls.

### AC-3 Dashboard and TUI convergence is explicitly defined
Given dashboard APIs and TUI operator workflows,
when the plan is inspected,
then it includes parity criteria, shared data contracts, and end-to-end operator flow acceptance gates.

### AC-4 Auth workflow validation is explicitly defined
Given gateway/auth/runtime contracts,
when the plan is inspected,
then it includes a complete lifecycle matrix for bootstrap/login/session use/expiry/logout/rotation/revocation/negative paths and required tests.

### AC-5 Release-readiness and rollback gates are explicitly defined
Given full-program delivery objectives,
when the plan is inspected,
then it includes pre-GA verification gates, operational checks, and rollback triggers for each critical stream.

### AC-6 Artifacts satisfy AGENTS process contract
Given repository process requirements,
when artifacts are checked,
then milestone index and issue `spec.md/plan.md/tasks.md` exist and are internally consistent (scope, AC mapping, conformance mapping, and test-tier expectations).

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
|---|---|---|---|---|---|
| C-01 | AC-1 | Conformance | milestone `M296` + issue tree | review decomposition section | ordered phases and dependencies are explicit |
| C-02 | AC-1 | Conformance | `specs/3424/tasks.md` | review task ordering | child workstreams are decomposed with ownership intent |
| C-03 | AC-2 | Functional/Conformance | RL stream definition | inspect phase plan | true RL gates include reward, optimizer, significance, persistence |
| C-04 | AC-2 | Conformance | verification matrix | inspect test-tier mapping | RL stream has deterministic validation commands |
| C-05 | AC-3 | Functional/Conformance | dashboard + TUI stream | inspect parity criteria | operator workflows are mapped end-to-end |
| C-06 | AC-3 | Integration | shared contract expectations | inspect plan interfaces | dashboard/TUI data contract convergence is defined |
| C-07 | AC-4 | Functional/Conformance | auth stream matrix | inspect lifecycle coverage | bootstrap/login/session/expiry/logout/rotation/revocation are covered |
| C-08 | AC-4 | Regression | negative-path auth cases | inspect validation matrix | unauthorized/expired/replayed/tampered scenarios are required |
| C-09 | AC-5 | Conformance | release gates | inspect phase 5/6 criteria | pre-GA checklist and rollback triggers are explicit |
| C-10 | AC-5 | Functional | operational readiness | inspect runbook requirements | observability and incident pathways are part of gate |
| C-11 | AC-6 | Conformance | repository tree | verify file presence | milestone + issue artifacts exist at expected paths |
| C-12 | AC-6 | Conformance | artifact cross-links | verify references | spec/plan/tasks and milestone references are coherent |

## Success Metrics / Observable Signals
- Milestone container exists: `specs/milestones/m296/index.md`.
- Binding artifact set exists: `specs/3424/spec.md`, `specs/3424/plan.md`, `specs/3424/tasks.md`.
- Plan explicitly defines delivery phases, acceptance gates, and dependency order for RL, dashboard/TUI, and auth.
- Program execution can start immediately via child-task creation without re-planning.
