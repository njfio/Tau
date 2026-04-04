# Deep Interview Spec: tau-application-improvement-audit

## Metadata
- Profile: `standard`
- Context type: `brownfield`
- Final ambiguity: `18%`
- Threshold: `<= 20%`
- Interview transcript: `.omx/interviews/tau-application-improvement-audit-20260403T040049Z.md`
- Context snapshot: `.omx/context/tau-application-improvement-audit-20260403T040049Z.md`

## Clarity Breakdown
| Dimension | Score |
|---|---:|
| Intent clarity | 0.85 |
| Outcome clarity | 0.92 |
| Scope clarity | 0.88 |
| Constraint clarity | 0.80 |
| Success criteria clarity | 0.72 |
| Brownfield context clarity | 0.80 |

## Intent
Produce a prioritized improvement roadmap for Tau, grounded in the current codebase, with the highest priority on agent autonomy, the next priority on product usefulness, and self-improvement/learning deferred until the autonomy baseline is real and trustworthy.

## Desired Outcome
Identify the most important missing connections and maturity gaps preventing Tau from reliably handling relatively complex, multi-step tasks with minimal human intervention.

## In Scope
- Brownfield assessment of what Tau already has connected vs still staged or fragmented
- Prioritized roadmap for autonomy-first improvement
- Gaps across runtime composition, mission control, operator flow, execution reliability, and adjacent product usefulness
- Evidence-backed notes on live testing and loop maturity

## Out of Scope / Non-Goals
- Multi-channel breadth as a first-tranche optimization target
- True self-learning / RL productionization in tranche one
- Dashboard polish unless it directly blocks autonomous task completion
- Broad developer-experience cleanup unless it directly blocks autonomous task completion
- General live provider expansion beyond the current Codex/OAuth path

## Decision Boundaries
- Acceptable human intervention in tranche one:
  - provider auth / approval clicks
  - checkpoints only for materially different solution directions
- Not acceptable for tranche one:
  - routine human steering between ordinary execution steps
  - requiring continuous operator supervision for normal progress

## Constraints
- The roadmap should optimize for:
  1. agent autonomy
  2. product usefulness
  3. self-improvement / learning
- The first autonomy milestone should not depend on broad multi-channel maturity
- The first autonomy milestone should preserve explicit checkpointing for major direction changes

## Testable Acceptance Criteria
1. The roadmap clearly distinguishes what is already connected and runnable today from what remains partial, staged, or only planned.
2. The roadmap orders work so tranche one primarily improves autonomous execution of complex tasks, not cosmetic UX or broad platform expansion.
3. The roadmap uses a measurable tranche-one success bar:
   - Tau completes a small benchmark set of `3-5` complex tasks
   - only auth and materially different solution-direction checkpoints require human intervention
4. The roadmap explicitly treats self-improvement/learning as a later multiplier, not the first autonomy milestone.

## Pressure-Pass Findings
- Early ambiguity: the user wanted “agent autonomy first” but had not defined what level of human intervention was still acceptable.
- Pressure result: tranche-one autonomy still allows operator checkpoints, but only for materially different solution directions.
- Early ambiguity: the roadmap risked ballooning into UX/DX polish.
- Pressure result: dashboard polish and broad DX cleanup are explicitly out unless they block autonomous task completion.
- Early ambiguity: “success” was aspirational rather than measurable.
- Pressure result: tranche one now has a concrete benchmark target of `3-5` complex tasks completed reliably with limited checkpoints.

## Assumptions Exposed + Resolutions
- Assumption: “autonomy first” might imply zero intervention.
  - Resolved: zero intervention is not required; major-direction checkpoints are acceptable.
- Assumption: multi-channel and self-learning might be part of tranche one.
  - Resolved: both are deferred.
- Assumption: dashboard and DX improvement might be roadmap priorities by default.
  - Resolved: both are deferred unless blocking autonomy.

## Brownfield Evidence vs Inference
- Evidence:
  - `README.md` describes multiple integrated end-to-end paths as runnable today, including local operator, gateway auth/session, unified runtime lifecycle, multi-channel ingress, prompt optimization, and a connected operator GA loop.
  - `README.md` also explicitly marks dashboard UX, broader RL/policy operations, live third-party validation, multi-channel live uptime, and full E2E scenario-group completion as partial or expansion-track areas.
  - `specs/milestones/m334/index.md` defines an active Ralph-loop supervisor milestone whose goal is a continuous outer supervisor above the inner agent/tool loop.
  - `docs/guides/training-ops.md` and `docs/planning/true-rl-roadmap-skeleton.md` show a deterministic RL and training proof stack with further production hardening still planned.
  - Gateway/TUI code under `crates/tau-gateway/src/gateway_openresponses/` and `crates/tau-tui/src/interactive/` already includes mission persistence, explicit completion/checkpoint semantics, mission APIs, and TUI resume controls.
  - `docs/plans/2026-03-23-001-feat-governed-mission-mode-plan.md` and `docs/plans/2026-03-23-005-feat-spec-to-pr-autopilot-plan.md` describe the missing product layer as mission composition and autopilot orchestration, not raw subsystem absence.
- Inference:
  - Tau is strong on deterministic proofs, runtime slices, and operator primitives, but still weaker on a default, reliable mission-level autonomous operating model for complex tasks.
  - The highest-leverage roadmap work is composition and default-path wiring, not adding more adjacent infrastructure first.

## Technical Context Findings
- Mission-level gateway/TUI slices exist:
  - persistent mission state
  - explicit `complete_task` semantics
  - mission list/detail endpoints
  - TUI mission resume and active mission binding
- Learning surfaces exist but still show partial integration / drift:
  - action history and learning bulletin primitives are present
  - some plans and ideation still describe the self-improvement loop as dormant or not fully wired end-to-end
  - command surfaces such as `/learn-status`, `/training-status`, and `/training-trigger` are present in command catalogs, but local evidence suggests they are still closer to planned/operator-surface scaffolding than a fully productized default loop
- Training/RL is operational as deterministic proof infrastructure, but broader production learning loops remain planned expansion work

## Condensed Transcript
1. Desired artifact: prioritized roadmap
2. Priority order: autonomy first, usefulness second, self-improvement third
3. Non-goals: multi-channel breadth and self-learning for tranche one; live provider scope stays narrow
4. Success shape: complex multi-step tasks like building a game, landing-page exploration, or dashboard-plus-presentation work
5. Human intervention: allowed only for major decisions
6. Major decisions: materially different solution directions
7. Dashboard polish and broad DX: out unless blocking autonomy
8. Tranche-one success bar: `3-5` complex tasks completed reliably with only auth and major-direction checkpoints
