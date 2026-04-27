# Tasks: Issue #3654 - Define the governed Tau Ralph supervisor loop across gateway, session, memory, and learning

- [ ] T1 Specify: publish the mission-supervisor loop contract, verifier
      contract, and state ownership boundaries across mission/session/memory.
- [ ] T2 Plan: break the architecture into implementation slices covering
      supervisor state, outer-loop execution, verifier adapters, memory/learning
      writeback, and operator surfaces.
- [ ] T3 Align: map existing Tau subsystems (`tau-session`, `tau-memory`,
      cortex, `tau-orchestrator`, gateway/TUI) into the loop and identify
      compatibility/migration boundaries.

## Implementation slice: mission completion outcome snapshots

- [x] T4 RED: add a gateway stream contract for mission completion outcome
      snapshots, covering `complete_task(status="partial")` as a
      `mission.checkpointed` operator event and `complete_task(status="blocked")`
      as a blocked operator snapshot.
- [x] T5 GREEN: thread completion-signal outcomes into the streamed
      `response.operator_turn_state.snapshot` payload without removing legacy
      `response.completed` compatibility.
- [ ] T6 CLOSEOUT: document and publish evidence that the first governed Ralph
      supervisor loop slice exposes checkpointed/blocked mission outcomes to
      operator surfaces.

Evidence:
- `cargo test -p tau-gateway mission_completion_outcome_snapshot -- --test-threads=1`
  proves streamed operator snapshots now expose mission completion outcome
  semantics for checkpointed and blocked governed-loop turns.
- `cargo test -p tau-gateway operator_turn_state_recovery_policy_snapshot -- --test-threads=1`
  keeps the #3673 verifier-blocked recovery policy snapshot path compatible
  with the mission completion outcome snapshot path.
