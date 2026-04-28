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

## Implementation slice: session memory learning handoff

- [x] T7 RED: add gateway coverage for a mission completion learning handoff,
                  proving checkpointed/blocked completion outcomes are written to
                  action-history learning records with session and mission identifiers.
- [x] T8 GREEN: persist completion outcome records into the existing
                  `tau-memory` action-history store so the next Ralph-loop iteration can
                  inject the outcome through the gateway learning bulletin.
- [ ] T9 CLOSEOUT: verify the handoff with the mission outcome snapshot guard,
                  document the session/memory/learning ownership boundary, and publish #3654
                  evidence.

Handoff boundary:
- Session: the OpenResponses session key remains the durable lineage anchor for
      action-history learning records.
- Memory: `tau-memory` action history remains the reusable learning store; this
      slice must not create a disconnected per-feature state file.
- Learning: checkpointed and blocked mission completion outcomes should become
      action-history learning inputs that can appear in the gateway learning bulletin
      for a later Ralph-loop iteration.

Evidence:
- `cargo test -p tau-gateway mission_completion_learning_handoff -- --test-threads=1`
      proves checkpointed and blocked `complete_task` outcomes are persisted as
      `complete_task` action-history records keyed by session and mission.
- `cargo test -p tau-gateway mission_completion_outcome_snapshot -- --test-threads=1`
      proves the learning handoff does not regress the streamed operator snapshot
      semantics for checkpointed and blocked mission outcomes.

## Implementation slice: verifier-blocked recovery learning handoff

- [x] T10 Specify: define verifier-blocked fail-closed recovery outcomes as
       reusable Ralph learning signals when gateway verifiers block no-tool fabricated
       progress or read-only-only implementation completion claims.
- [x] T11 RED: add gateway coverage proving a verifier-blocked fabricated-progress
       mission writes an unsuccessful action-history record with the verifier reason code,
       session key, and mission id.
- [x] T12 GREEN: persist verifier-blocked recovery records into the existing
       `tau-memory` action-history store so future learning bulletins can warn against
       repeating no-tool or missing-mutation completion claims.
- [x] T13 CLOSEOUT: verify the new verifier-blocked learning handoff alongside
       fabricated-progress blocking, mutating-evidence blocking, existing mission
       completion learning, formatting, clippy, and Cargo manifest stability.

Verifier-blocked handoff boundary:
- Session: the OpenResponses session key remains the durable lineage anchor for
      verifier-blocked action-history learning records.
- Memory: `tau-memory` action history remains the shared learning store; this
      slice must not introduce a separate verifier-specific learning file.
- Learning: fail-closed verifier outcomes should become unsuccessful action-history
      records that preserve the verifier reason code and mission id so a later
      Ralph-loop iteration can learn that assistant-only or read-only-only completion
      claims were blocked.
- Operator rows: verifier-blocked learning records must not reintroduce
      `complete_task` or verifier internals into normal observed tool rows; they are
      learning evidence, not user-visible tool execution evidence.

Evidence:
- `cargo test -p tau-gateway verifier_blocked_learning -- --test-threads=1`
      proves verifier-blocked fabricated-progress outcomes are persisted as unsuccessful
      `gateway_verifier` action-history learning records keyed by session and mission,
      without creating `complete_task` rows.
- `cargo test -p tau-gateway fabricated_progress -- --test-threads=1`
      keeps the #3602 no-tool fabricated-progress fail-closed policy compatible with
      verifier-blocked learning records.
- `cargo test -p tau-gateway mutating_tool_evidence -- --test-threads=1`
      keeps the #3603 read-only-only missing-mutation fail-closed policy compatible with
      verifier-blocked learning records.
- `cargo test -p tau-gateway mission_completion_learning_handoff -- --test-threads=1`
      keeps normal `complete_task` completion learning separated from verifier-blocked
      recovery learning evidence.
- `cargo fmt --check`, `cargo clippy -p tau-gateway --tests --no-deps -- -D warnings`,
      and `git diff --quiet -- Cargo.toml` passed for the slice.
