# feat: Tau Agent Harness Lane

Status: Active
Date: 2026-05-03
Milestone alignment: `specs/milestones/m334/index.md`
Related plans:
- `docs/plans/2026-03-23-001-feat-governed-mission-mode-plan.md`
- `docs/plans/2026-03-23-004-feat-self-improvement-engine-plan.md`
- `docs/plans/2026-04-03-001-feat-tranche-one-autonomy-vertical-slice-plan.md`
- `docs/plans/2026-04-03-002-deepened-state-of-the-art-autonomous-agent-plan.md`

## Intent

Define a first-class Tau Agent Harness product lane for a self-improving autonomous engineer and personal agent.
This lane should be judged by mission completion quality, recoverability, verification evidence, and learning quality rather than by the number of gateways, channels, or dashboards it exposes.

The product center is:

- `tau-agent-core`
- `tau-coding-agent`
- `tau-tools`
- `tau-session`
- `tau-memory`
- `tau-skills`
- `tau-safety`
- `tau-orchestrator`

Gateway, channel, dashboard, and UI surfaces are adapters. They may start missions, inspect missions, approve mission actions, and render mission state, but they should not own mission truth, duplicated state machines, or product direction.

## Product Boundary

Tau's moat should be a dependable autonomous harness that can plan, execute, verify, recover, and improve across coding, research, data, and personal-assistant tasks.
OpenClaw/Hermes-style gateway breadth is useful as an adapter strategy, but it is not the core product goal for this lane.

In scope:

- Durable mission state as the top-level unit of work.
- Plan DAG execution with explicit checkpoints and recovery.
- Tool budgeting and tool-use proof.
- Memory recall, memory writeback, and learning records.
- Safety-reviewed skill, config, and prompt improvement.
- Autonomy benchmark fixtures and scorer.
- Adapter projections for gateway/channel/dashboard consumers.

Out of scope for the first lane:

- Expanding channel count as a success metric.
- Marketplace or broad connector strategy.
- Silent self-modification of Rust source code.
- Safety-policy mutation by the agent itself.
- Dashboard-first state ownership.

## Current Ownership Gap

The repository already has several harness primitives, but mission ownership is still too adapter-shaped for the product goal.
The current gateway runtime owns mission-supervisor state in `crates/tau-gateway/src/gateway_openresponses/mission_supervisor_runtime.rs`, while orchestration remains generic in `crates/tau-orchestrator/src/plan.rs` and `crates/tau-orchestrator/src/plan_executor.rs`, and `tau-coding-agent` acts as the primary integration glue through `crates/tau-coding-agent/src/orchestrator_bridge.rs`.

That was a reasonable first slice because the gateway exposed visible operator workflows early.
The harness lane now needs an ownership inversion:

- Gateway-local mission state migrates into shared harness-owned mission types.
- Existing gateway mission endpoints stay compatible by serializing shared mission views.
- `tau-coding-agent` becomes a runner over the shared mission contract instead of the only durable glue point.
- `tau-orchestrator` schedules plan DAG work against a mission identity, not against anonymous plan execution.
- Adapter surfaces never infer success from assistant prose when shared verifier/completion state is available.

This inversion should be captured in an ADR before broad implementation because it intentionally supersedes the gateway-centric assumption in the first governed-mission and tranche-one autonomy slices.

## Mission Contract

`Mission` becomes the top-level durable unit, above chat turns, gateway requests, or isolated tool calls.
The canonical mission model should live in the harness lane, with adapter-specific DTOs derived from it.

Minimum mission fields:

| Field | Purpose |
| --- | --- |
| `mission_id` | Stable durable identity across restarts, adapters, and recovery. |
| `goal` | Operator-stated objective and bounded scope. |
| `acceptance_criteria` | Testable ACs, each mapped to verification gates. |
| `plan_dag` | Ordered and dependency-aware tasks, including blocked and skipped nodes. |
| `tool_budget` | Allowed tools, maximum calls/time/cost, and budget consumption. |
| `memory_hits` | Recall evidence used to shape the plan or execution. |
| `verification_gates` | Commands, assertions, human checks, and artifact checks required for completion. |
| `checkpoints` | Durable restart points with summaries, artifacts, and pending work. |
| `recovery_state` | Blockers, retries, fallback path, last good checkpoint, and resume instructions. |
| `artifacts` | Files, docs, patches, reports, screenshots, datasets, PRs, and generated outputs. |
| `final_learning_output` | Structured lessons, curator recommendations, and next-run improvements. |

Mission lifecycle:

1. `Draft` - goal captured, ACs incomplete or unaccepted.
2. `Planned` - ACs, plan DAG, tool budget, and verification gates exist.
3. `Executing` - tool calls, edits, or external actions are underway.
4. `Checkpointed` - safe durable resume point written.
5. `Blocked` - no safe next step without new authority or input.
6. `Verifying` - completion proof is being collected.
7. `Completed` - ACs passed and final learning output written.
8. `Failed` - recovery exhausted or mission invalidated.
9. `Archived` - retained as benchmark/training/reference material.

Ownership split:

- `tau-agent-core`: mission domain types, lifecycle invariants, completion semantics.
- `tau-orchestrator`: plan DAG scheduling, dependency readiness, checkpoint/resume flow.
- `tau-coding-agent`: coding mission execution loop and evidence capture.
- `tau-tools`: normalized tool-call records and budget accounting.
- `tau-session`: transcript, run lineage, and operator approval linkage.
- `tau-memory`: memory hits, learning records, curator queues.
- `tau-skills`: executable skill/config/prompt package surface.
- `tau-safety`: policy gates, approval requirements, deny rules, trust boundaries.
- Gateway/channel/dashboard adapters: mission creation, projection, approval, and observation.

## Conservative Self-Improvement Loop

The first self-improvement loop is intentionally limited to safe skill, config, and prompt changes.
It should not auto-apply source-code changes or mutate safety policy.

Required loop:

1. Observe failure: verifier block, operator correction, benchmark miss, repeated tool failure, or recovery fallback.
2. Write learning record: persist failure context, root cause, evidence, and affected mission IDs.
3. Synthesize patch: generate a skill, config, or prompt change proposal with rationale and rollback plan.
4. Dry-run: apply the patch in an isolated simulation or preview mode and record expected effects.
5. Test: run targeted tests, benchmark replay, and safety checks before eligibility.
6. Operator-reviewed apply: require explicit approval before updating active skill/config/prompt state.
7. Curator update: record the accepted learning in memory and skill/config metadata for future recall.

Hard guardrails:

- No silent apply.
- No direct safety-policy relaxation.
- No credential, secret, or trust-root modification.
- No Rust source self-edit in the first lane.
- Every accepted improvement must include mission evidence, test evidence, approval evidence, and rollback instructions.

## Canonical Autonomy Benchmark

Add a Tau autonomy benchmark that measures whether the harness can complete full missions with proof.
The benchmark should be small enough to run regularly but broad enough to prevent overfitting to coding-only tasks.

Benchmark task classes:

| Class | Example mission | Required proof |
| --- | --- | --- |
| `repo_bugfix` | Fix a scoped bug in an existing repo. | Failing test first, patch, green test, memory write, learning output. |
| `greenfield_utility` | Build a small CLI/library utility from a prompt. | ACs, plan DAG, generated artifact, tests, usage proof. |
| `research_to_doc` | Research a bounded topic and write a repo doc/spec. | Source notes, citations or local evidence, doc diff, review checklist. |
| `data_to_deliverable` | Convert structured input into a report or artifact. | Data validation, transformation trace, final artifact, verification checks. |
| `personal_assistant_automation` | Schedule, monitor, summarize, or organize a personal workflow. | Intent capture, tool execution trace, confirmation policy, recovery path. |

Each benchmark run must emit:

- Mission record with goal, ACs, and plan DAG.
- Tool trace with budget use.
- Memory read and memory write evidence.
- Verification gate results.
- Artifact manifest.
- Operator intervention count.
- Final learning output.
- Pass/fail score plus explanation.

Initial pass criteria:

- Every task class has at least one fixture.
- The runner rejects missions without planning proof, tool proof, memory proof, verification proof, or learning proof.
- Benchmark results can be compared across runs without relying on gateway-specific state.
- Failures create learning records rather than disappearing into logs.

## Delivery Slices

### Slice 1: Lane Spec and Mission Schema

- Create or update an issue-bound spec under `specs/<issue-id>/`.
- Add an ADR under `docs/architecture/` for mission ownership inversion.
- Define mission schema, state machine, invariants, and adapter projection rules.
- Add schema/unit tests before implementation.

Acceptance criteria:

- AC-1: Mission schema contains all required durable fields.
- AC-2: Mission lifecycle rejects invalid transitions.
- AC-3: Gateway/channel/dashboard code consumes projections rather than owning canonical mission state.
- AC-4: The ADR names the old gateway-centric assumption and the new harness-owned mission boundary.

### Slice 2: Extract Shared Mission State

- Promote gateway-local `Running`, `Completed`, `Checkpointed`, and `Blocked` concepts into shared harness-owned mission types.
- Represent session linkage, verifier state, completion state, artifacts, attempt history, and recovery state without HTTP assumptions.
- Keep gateway APIs compatible by delegating to shared mission services and adapter DTOs.

Acceptance criteria:

- AC-1: Mission state can be persisted and loaded without importing gateway HTTP modules.
- AC-2: Gateway mission endpoints serialize shared mission views rather than private gateway-only state.
- AC-3: Existing gateway behavior has compatibility tests before migration.

### Slice 3: Plan DAG and Checkpoint Runtime

- Bind mission plans to an explicit DAG.
- Persist node state, dependencies, skipped nodes, retries, and checkpoint summaries.
- Add resume tests from checkpointed and blocked states.

Acceptance criteria:

- AC-1: A mission can resume after process restart without losing plan state.
- AC-2: Blocked nodes preserve blocker reason and next recovery action.
- AC-3: Completion requires all required verification gates to pass.

### Slice 4: Tool Budget and Evidence Ledger

- Normalize tool-call evidence through `tau-tools`.
- Track budget limits and consumption per mission.
- Attach evidence to mission artifacts and verification gates.

Acceptance criteria:

- AC-1: Tool calls are attributable to mission ID and plan node ID.
- AC-2: Budget exhaustion blocks further autonomous tool execution.
- AC-3: Completion reports include tool trace evidence.

### Slice 5: Memory and Learning Records

- Attach memory hits to plan rationale.
- Write final learning output and failure learning records through `tau-memory`.
- Add curator queue status for later operator review.

Acceptance criteria:

- AC-1: Mission planning records relevant memory hits or an explicit no-memory result.
- AC-2: Mission completion writes a final learning record.
- AC-3: Failure recovery writes a learning record before proposing improvement.

### Slice 6: Conservative Self-Improvement

- Implement the observe-to-curator loop for skill/config/prompt proposals only.
- Reuse existing dry-run/self-modification preview boundaries where possible.
- Require operator approval before active application.

Acceptance criteria:

- AC-1: A benchmark failure can produce a dry-run skill/config/prompt proposal.
- AC-2: Tests and safety checks are recorded before approval eligibility.
- AC-3: Apply is impossible without explicit operator approval.
- AC-4: Accepted improvements update memory/curator metadata.

### Slice 7: Autonomy Benchmark

- Add benchmark fixtures for all canonical classes.
- Add a scorer that requires planning, tool, memory, verification, artifact, and learning proof.
- Use benchmark results as the release gate for the harness lane.

Acceptance criteria:

- AC-1: Each task class has a deterministic fixture.
- AC-2: Missing proof fails the benchmark.
- AC-3: Results identify regressions by mission field, not only by final success.

### Slice 8: Adapter Realignment

- Audit gateway/channel/dashboard code for duplicated mission concepts.
- Convert adapter surfaces to mission creation/projection/approval clients.
- Keep dashboard/operator views focused on mission state, evidence, and approvals.

Acceptance criteria:

- AC-1: Adapter surfaces do not define independent mission state machines.
- AC-2: Operator approval events link back to mission IDs and improvement proposals.
- AC-3: Product docs describe adapters as surfaces, not the core lane.

## Verification Strategy

For the planning artifact:

- Markdown diff review.
- Link/path sanity check against existing docs and milestone files.

For implementation slices:

- `cargo fmt --check`
- Targeted crate tests for touched crates.
- Mission lifecycle unit tests.
- Plan DAG resume tests.
- Tool-budget exhaustion tests.
- Memory/learning record tests.
- Self-improvement dry-run and approval-gate tests.
- Autonomy benchmark fixture run.

## Risks and Controls

| Risk | Control |
| --- | --- |
| Gateway breadth consumes product focus. | Treat gateway/channel/dashboard as adapters and score the lane by benchmarked mission completion. |
| Mission truth fragments across crates. | Put canonical domain/state invariants in the harness lane and expose adapter DTOs only. |
| Self-improvement becomes unsafe source mutation. | Start with skill/config/prompt proposals only, dry-run first, operator approval required. |
| Benchmark becomes theater. | Require proof fields and reject runs missing planning, tool, memory, verification, artifact, or learning evidence. |
| Planning docs drift from implementation. | Bind next work to issue specs under `specs/<issue-id>/` and M334 task tracking before code changes. |

## Next Issue Shape

Create or select an M334-aligned issue for "Tau Agent Harness mission contract and benchmark foundation."
The first implementation issue should be P1 because it crosses core runtime, orchestration, memory, tools, skills, and safety boundaries.
It should produce `specs/<issue-id>/spec.md`, `plan.md`, and `tasks.md` before code changes.

Suggested first task order:

1. Write the issue spec and self-accept/review according to the repo contract.
2. Add mission schema tests.
3. Add lifecycle transition tests.
4. Add adapter projection tests.
5. Implement the minimum mission model.
6. Wire one coding-agent mission path.
7. Add one benchmark fixture that exercises the whole proof contract.
