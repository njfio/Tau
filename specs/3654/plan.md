# Plan: Issue #3654 - Define the governed Tau Ralph supervisor loop across gateway, session, memory, and learning

## Approach
1. Define a top-level `MissionSupervisor` model that wraps the existing inner
   `Agent`/tool loop instead of replacing it.
2. Model the canonical Tau loop as:
   - load mission + current plan/task + prior learning context
   - run one inner execution attempt
   - run verifier/back-pressure evaluation
   - record mission/session/action outcomes
   - distill memory + refresh learning/cortex context
   - continue, replan, checkpoint, block, or complete
3. Reuse existing Tau subsystems as follows:
   - `tau-session`: durable lineage, branching, resume, undo/redo
   - `tau-memory/action_history`: per-turn outcomes, tool success/failure
     patterns, session feedback
   - cortex runtime: operator-visible distilled learning bulletin
   - `tau-orchestrator`: active plan/task progression and reporting
   - gateway/TUI: operator controls and mission status views
4. Add an explicit completion/checkpoint contract to the outer loop:
   - completion signal/tool for "objective satisfied"
   - blocked/approval-needed/partial-progress checkpoint states
   - bounded retry/replan policies
5. Keep current prompt/session flows on an implicit single-mission compatibility
   path while migrating primary operator entrypoints to the supervisor model.

## Proposed Architecture
### Loop Kernel
- `MissionSupervisor` owns mission id, goal, acceptance criteria, current task,
  retry/replan budget, checkpoint state, verifier status, and linked session +
  memory identifiers.
- The inner agent remains responsible for tool use and local reasoning.
- The outer supervisor remains responsible for deciding whether another
  iteration is required.

### Back-Pressure
- Verifiers are first-class inputs, not post-hoc logs:
  build/test/lint/file existence/screenshot/e2e/policy checks.
- Each iteration produces structured verifier outcomes that feed the next
  prompt/context and operator UI.

### Memory + Learning
- Action history becomes default loop telemetry, not optional side storage.
- Session feedback and failure patterns are refreshed per mission iteration or
  per mission stage.
- Cortex bulletins summarize recent patterns and inject them into the next loop.
- Distilled mission memory records the current objective, recent failures,
  successful strategies, and unresolved blockers.

### Operator Controls
- Gateway/TUI expose one mission object with:
  current objective, active task, verifier state, checkpoint status, retry
  count, latest artifacts, and learning summary.
- Operators can pause/resume/retry/replan/approve from that shared state.

## Suggested Implementation Slices
1. Mission supervisor state model and persistence
2. Outer-loop executor around current gateway/TUI prompt path
3. Verifier contract and first verifier adapters
4. Memory/learning writeback + context injection
5. Operator mission views and controls
6. Compatibility migration for legacy prompt/session entrypoints

## Affected Areas
- `crates/tau-gateway`
- `crates/tau-agent-core`
- `crates/tau-session`
- `crates/tau-memory`
- `crates/tau-orchestrator`
- `crates/tau-tui`

## Risks / Mitigations
- Risk: mission, session, and memory state drift apart.
  Mitigation: define one canonical mission identifier and explicit write order.
- Risk: verifier feedback becomes another opaque text blob.
  Mitigation: require structured verifier outputs with reason codes and machine-
  readable fields.
- Risk: learning signals stay passive and do not affect behavior.
  Mitigation: define mandatory context injection of failure patterns/tool
  effectiveness into each next iteration.
- Risk: migration breaks prompt-only entrypoints.
  Mitigation: keep an implicit single-mission compatibility path during rollout.

## Verification
- Spec review confirms one canonical outer loop and state ownership model
- Follow-up stories are sliced so each can be implemented/tested independently
- Architecture reuses existing Tau subsystems instead of inventing parallel
  persistence/control planes
