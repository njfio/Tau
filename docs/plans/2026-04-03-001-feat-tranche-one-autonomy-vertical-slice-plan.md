---
title: feat: Ship Tranche-One Autonomy Vertical Slice
type: feat
status: active
date: 2026-04-03
origin: .omx/specs/deep-interview-tau-application-improvement-audit.md
---

# feat: Ship Tranche-One Autonomy Vertical Slice

## Overview

Define and deliver the first executable autonomy milestone for Tau: a narrow but real vertical slice that proves Tau can complete a small benchmark set of `3-5` relatively complex tasks with only provider-auth friction and operator checkpoints for materially different solution directions.

This plan treats the benchmark itself as a product requirement, not an afterthought. Tau already has many integrated subsystems and deterministic verification gates, but it does not yet show a clear, user-relevant proof that the system can autonomously carry a complex mission to completion in the way the product now needs to promise.

## Problem Statement / Motivation

Tau has substantial infrastructure already in place:

- integrated CLI, gateway, TUI, and operator workflows in [README.md](../../README.md)
- Ralph-loop mission persistence, verifier bundles, explicit completion semantics, and TUI resume controls in the gateway/TUI runtime
- deterministic RL, training, and operator verification harnesses in [training-ops.md](../guides/training-ops.md) and [true-rl-roadmap-skeleton.md](true-rl-roadmap-skeleton.md)
- existing product-direction plans for governed mission mode, TUI mission control, and spec-to-PR autopilot in:
  - [2026-03-23-001-feat-governed-mission-mode-plan.md](2026-03-23-001-feat-governed-mission-mode-plan.md)
  - [2026-03-23-002-feat-tui-mission-control-plan.md](2026-03-23-002-feat-tui-mission-control-plan.md)
  - [2026-03-23-005-feat-spec-to-pr-autopilot-plan.md](2026-03-23-005-feat-spec-to-pr-autopilot-plan.md)

The missing layer is composition and proof. Tau can demonstrate many deterministic loops, but it does not yet present or verify a canonical autonomy path where one governed mission object drives planning, execution, verifier-backed continuation, checkpointing, resume, and operator steering across a complex task benchmark (see origin: `.omx/specs/deep-interview-tau-application-improvement-audit.md`).

## Proposed Solution

Deliver a tranche-one autonomy vertical slice in four connected parts:

1. Define the benchmark:
   - create a durable benchmark set of `3-5` complex tasks representing the target product bar
   - include at least one build task, one research/design task, and one data-to-deliverable task
   - encode the allowed intervention model: auth friction plus major-direction checkpoints only

2. Bind the benchmark to one canonical mission path:
   - use the Ralph-loop mission model as the default execution object for the slice
   - ensure benchmark runs produce one inspectable mission identifier with session, verifier, checkpoint, and artifact linkage

3. Productize one useful autonomy domain:
   - choose a bounded mission domain where Tau already has strong adjacent primitives
   - the best candidate is spec-to-PR autopilot because the repo already enforces a strong issue/spec/plan/task workflow
   - benchmark at least one task through this domain end-to-end

4. Expose the slice clearly to the operator:
   - show benchmark mission state, verifier pressure, checkpoints, and recovery state from existing TUI/gateway surfaces
   - avoid broad dashboard or DX cleanup unless directly required for this slice

## Key Technical Decisions

### 1. Use an executable benchmark as the tranche-one contract

Decision:
- define benchmark missions in both human-readable and machine-readable form
- require mission-level result classification rather than prose-only success claims

Rationale:
- Tau already has many deterministic subsystem proofs, so the missing proof is user-task autonomy, not another narrative status document

### 2. Reuse existing gateway Ralph-loop mission state for the first slice

Decision:
- use the existing gateway mission object as the canonical runtime identity for the vertical slice
- defer broader mission-state unification until the benchmark shows what is still missing

Rationale:
- mission persistence, completion/checkpoint semantics, and TUI resume already exist in code
- creating a second autonomy object now would increase drift and make the benchmark less trustworthy

### 3. Start with one bounded usefulness domain: spec-to-PR autopilot

Decision:
- productize one benchmark task through the spec-to-PR/autopilot direction before attempting general autonomy

Rationale:
- this repo already has strong governance, issue/spec/task structure, and adjacent GitHub/runtime surfaces
- it gives Tau a useful, measurable autonomy domain that fits the codebase instead of forcing an abstract “general intelligence” proof

### 4. Keep checkpoints only for materially different solution directions

Decision:
- ordinary execution, retries, and verifier loops remain autonomous
- explicit operator approval is reserved for major design forks or external dependency boundaries

Rationale:
- this matches the origin interview’s clarified boundary
- it protects trust without reducing the slice to human-driven wizardry

## Technical Approach

### Architecture

The vertical slice should build on existing runtime pieces rather than create parallel abstractions:

- Mission state:
  - [`crates/tau-gateway/src/gateway_openresponses/mission_supervisor_runtime.rs`](../../crates/tau-gateway/src/gateway_openresponses/mission_supervisor_runtime.rs)
  - [`crates/tau-gateway/src/gateway_openresponses/mission_api_runtime.rs`](../../crates/tau-gateway/src/gateway_openresponses/mission_api_runtime.rs)
- Explicit completion and checkpoint semantics:
  - [`crates/tau-gateway/src/gateway_openresponses/mission_completion_runtime.rs`](../../crates/tau-gateway/src/gateway_openresponses/mission_completion_runtime.rs)
- TUI mission binding and resume:
  - [`crates/tau-tui/src/interactive/app.rs`](../../crates/tau-tui/src/interactive/app.rs)
  - [`crates/tau-tui/src/interactive/app_commands.rs`](../../crates/tau-tui/src/interactive/app_commands.rs)
  - [`crates/tau-tui/src/interactive/app_gateway_tests.rs`](../../crates/tau-tui/src/interactive/app_gateway_tests.rs)
- Orchestration / execution structure:
  - [`crates/tau-orchestrator`](../../crates/tau-orchestrator)
- Governance-aware autopilot direction:
  - [`docs/plans/2026-03-23-005-feat-spec-to-pr-autopilot-plan.md`](2026-03-23-005-feat-spec-to-pr-autopilot-plan.md)

The benchmark should not be defined as pure documentation. It needs executable evidence: fixtures, scenario definitions, mission-status assertions, and outcome checks that are meaningful for the selected task domains.

### Implementation Units

#### Unit 1: Benchmark schema and corpus

Create the tranche-one benchmark definition and task corpus.

Likely file targets:
- `docs/guides/` or `docs/planning/` for benchmark definition
- `tasks/fixtures/` or a benchmark-specific fixture directory for machine-readable mission cases
- report artifacts under `tasks/reports/` or a mission-local runtime report path

Verification outcomes:
- benchmark tasks are versioned and reviewable
- each task encodes deliverables, allowed checkpoint classes, and mission-level pass/fail semantics

#### Unit 2: Mission result classification adapter

Convert existing mission persistence and verifier data into benchmark-level results.

Likely file targets:
- `crates/tau-gateway/src/gateway_openresponses/openresponses_execution_handler.rs`
- `crates/tau-gateway/src/gateway_openresponses/mission_supervisor_runtime.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`

Verification outcomes:
- benchmark runs classify cleanly as `completed`, `checkpoint_required`, `blocked`, or `runtime_failed`
- classification is derived from mission/verifier/completion state rather than brittle string matching

#### Unit 3: Bounded autopilot execution path

Execute one benchmark case through a real autonomy domain.

Likely file targets:
- `crates/tau-github-issues`
- `crates/tau-github-issues-runtime`
- `crates/tau-orchestrator`
- `crates/tau-coding-agent`
- supporting docs/spec artifacts for governed issue/spec/task generation

Verification outcomes:
- one benchmark mission produces real governance artifacts end-to-end
- blocked states remain explicit when review or acceptance thresholds apply

#### Unit 4: Operator inspection and resume path

Ensure benchmark missions are inspectable and resumable from existing operator surfaces.

Likely file targets:
- `crates/tau-gateway/src/gateway_openresponses/mission_api_runtime.rs`
- `crates/tau-tui/src/interactive/app.rs`
- `crates/tau-tui/src/interactive/app_commands.rs`
- `crates/tau-tui/src/interactive/ui_status.rs`
- `crates/tau-tui/src/interactive/app_gateway_tests.rs`

Verification outcomes:
- benchmark mission status and next operator action are visible without raw file inspection
- resume preserves mission identity and session linkage

#### Unit 5: Benchmark evidence and closeout reporting

Summarize benchmark outcomes in a durable operator/maintainer report.

Likely file targets:
- `docs/guides/` for operator guidance
- `docs/planning/` for roadmap tracking
- report artifacts under `tasks/reports/` or a benchmark-specific artifact directory

Verification outcomes:
- maintainers can see which benchmark tasks passed, where checkpoints occurred, and where mission composition is still fragmented

### Implementation Phases

#### Phase 1: Benchmark Definition and Harness

- Create a tranche-one benchmark artifact set under `docs/planning/` or `docs/guides/` plus machine-readable benchmark fixtures under a repo-local runtime or fixture path.
- Define each benchmark task with:
  - mission goal
  - expected artifact/output shape
  - allowed human checkpoints
  - pass/fail verifier expectations
- Add a benchmark runner or validation harness that can report mission-level results rather than just low-level deterministic contract success.

#### Phase 2: Mission Path Consolidation

- Ensure the benchmark path always runs through one canonical mission identity.
- Verify mission/session/checkpoint/artifact linkage across gateway and TUI resume flows.
- Tighten verifier-driven continuation so benchmark missions cannot “succeed” on assistant text alone.

#### Phase 3: Bounded Autopilot Domain

- Implement one autonomy domain with strong usefulness and clear repo fit.
- Preferred domain:
  - spec-to-PR autopilot using the repo’s own `AGENTS.md` contract, issue templates, milestone/spec/task flow, and PR evidence structure
- The slice must halt cleanly at major-direction checkpoints instead of silently requiring routine human steering.

#### Phase 4: Operator Supervision and Evidence

- Surface benchmark mission status, blocked reasons, verifier state, active checkpoint, and next operator action in TUI/gateway flows.
- Add a simple evidence/report output showing:
  - completed benchmarks
  - blocked benchmarks
  - where checkpoints were required
  - where mission identity, verifier, or operator state was missing

## Alternative Approaches Considered

### 1. Start with self-improvement / RL first

Rejected for tranche one. The repo has strong deterministic training and RL proof infrastructure, but the deep-interview origin and existing plans both indicate that autonomy and usefulness need to be real before self-improvement becomes valuable (see origin: `.omx/specs/deep-interview-tau-application-improvement-audit.md`).

### 2. Start with broad dashboard or DX cleanup

Rejected for tranche one. Dashboard and DX work are explicitly out of scope unless they block autonomous task completion. Existing plans also describe those areas as secondary composition/polish layers rather than the main autonomy blocker.

### 3. Expand multi-channel breadth first

Rejected for tranche one. README and planning docs repeatedly describe multi-channel live behavior as environment-specific or expansion-track work, while the interview explicitly deprioritized it.

## System-Wide Impact

### Interaction Graph

Benchmark mission intake should create or select a mission, which drives the Ralph supervisor loop, which invokes the inner agent/tool loop, which records verifier outcomes and completion/checkpoint state, which updates gateway mission APIs, which feed TUI mission supervision. For the autopilot slice, this also touches tracker/spec/plan/task/PR artifact generation paths.

### Error & Failure Propagation

The benchmark harness must distinguish:

- auth/provider friction
- explicit blocked mission outcomes
- verifier retry exhaustion
- operator-required major-direction checkpoints
- infrastructure/runtime failures

These cannot collapse into one generic “mission failed” outcome or the benchmark becomes uninformative.

### State Lifecycle Risks

- Mission state can drift from session, artifacts, or operator-visible status if benchmark runs use multiple code paths.
- Benchmark fixtures can become stale if they do not track the actual mission/autopilot surfaces.
- Partial autopilot artifact generation can falsely look successful unless blocked state is first-class.

Mitigations:
- derive benchmark outcomes from mission persistence as close to the runtime source of truth as possible
- version benchmark fixtures and reports explicitly
- treat partial artifact generation as failure unless the mission records an allowed checkpointed or blocked outcome

### API Surface Parity

The vertical slice should align CLI/gateway/TUI around the same mission semantics. The TUI must not invent privileged mission states that the gateway cannot inspect or resume.

### Integration Test Scenarios

1. Start a benchmark mission, hit a verifier-backed retry, then complete successfully and verify one coherent mission record plus expected artifacts.
2. Start a benchmark mission that reaches a major-direction fork, require an operator checkpoint, resume after approval, and verify the same mission id continues.
3. Start a benchmark mission that blocks on an external dependency, return a checkpointed or blocked state, and verify operator surfaces show the reason and next action.
4. Run the chosen autopilot slice end-to-end and verify generated artifacts plus mission state stay consistent.
5. Resume an interrupted benchmark mission from TUI and verify session, mission, and verifier context are preserved.

## Acceptance Criteria

- [ ] A durable tranche-one autonomy benchmark exists for `3-5` complex tasks with explicit pass/fail criteria and checkpoint rules.
- [ ] At least one benchmark path runs through a canonical Ralph-loop mission identity with verifier-backed continuation and explicit checkpoint/completion state.
- [ ] One bounded autonomy domain is productized enough to execute a benchmark task end-to-end with only auth and major-direction checkpoints.
- [ ] Gateway/TUI operator surfaces can inspect benchmark mission status, blocked reasons, and resume/checkpoint state without direct file inspection.
- [ ] Benchmark reports clearly distinguish success, blocked, runtime failure, and checkpoint-required outcomes.
- [ ] The slice does not depend on broad multi-channel expansion, broad dashboard polish, or tranche-one self-learning rollout.

## Success Metrics

- Tau can complete a small benchmark set of `3-5` complex tasks with only auth and major-direction checkpoints.
- Operators can explain why a benchmark mission continued, blocked, or completed using mission/verifier evidence rather than chat reconstruction.
- The repo gains a repeatable benchmark that can drive future autonomy and self-improvement work instead of relying on anecdotal demos.

## Dependencies & Risks

### Dependencies

- Existing Ralph-loop gateway mission slices under milestone M334
- Governed mission mode direction from [2026-03-23-001-feat-governed-mission-mode-plan.md](2026-03-23-001-feat-governed-mission-mode-plan.md)
- TUI mission-control direction from [2026-03-23-002-feat-tui-mission-control-plan.md](2026-03-23-002-feat-tui-mission-control-plan.md)
- Spec-to-PR autopilot direction from [2026-03-23-005-feat-spec-to-pr-autopilot-plan.md](2026-03-23-005-feat-spec-to-pr-autopilot-plan.md)

### Risks

- The benchmark can become too synthetic and measure harness compliance rather than user-relevant autonomy.
- The slice can become too broad if it tries to solve full mission mode, full TUI mission control, and full autopilot simultaneously.
- Existing docs/plans show some drift around learning/runtime readiness; the benchmark must use code-path reality, not plan optimism.

### Sequencing Notes

- Define the benchmark before broad execution changes so the success bar stays stable.
- Land mission-result classification before richer operator UX so UI work reflects real autonomy state.
- Keep self-improvement out of the tranche-one critical path; it should consume benchmark evidence later, not block the benchmark from existing now.

## Sources & References

- **Origin document:** [.omx/specs/deep-interview-tau-application-improvement-audit.md](../../.omx/specs/deep-interview-tau-application-improvement-audit.md)
- Runtime status and capability boundaries: [README.md](../../README.md)
- Ralph-loop milestone: [specs/milestones/m334/index.md](../../specs/milestones/m334/index.md)
- Mission persistence: [mission_supervisor_runtime.rs](../../crates/tau-gateway/src/gateway_openresponses/mission_supervisor_runtime.rs)
- Mission API: [mission_api_runtime.rs](../../crates/tau-gateway/src/gateway_openresponses/mission_api_runtime.rs)
- Mission completion semantics: [mission_completion_runtime.rs](../../crates/tau-gateway/src/gateway_openresponses/mission_completion_runtime.rs)
- TUI mission resume path: [app.rs](../../crates/tau-tui/src/interactive/app.rs)
- TUI mission tests: [app_gateway_tests.rs](../../crates/tau-tui/src/interactive/app_gateway_tests.rs)
- Existing mission-mode plan: [2026-03-23-001-feat-governed-mission-mode-plan.md](2026-03-23-001-feat-governed-mission-mode-plan.md)
- Existing autopilot plan: [2026-03-23-005-feat-spec-to-pr-autopilot-plan.md](2026-03-23-005-feat-spec-to-pr-autopilot-plan.md)
- Existing TUI mission-control plan: [2026-03-23-002-feat-tui-mission-control-plan.md](2026-03-23-002-feat-tui-mission-control-plan.md)
- RL/training operations context: [training-ops.md](../guides/training-ops.md)
