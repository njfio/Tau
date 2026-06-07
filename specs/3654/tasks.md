# Tasks: Issue #3654 - Define the governed Tau Ralph supervisor loop across gateway, session, memory, and learning

- [x] T1 Specify: publish the mission-supervisor loop contract, verifier
      contract, and state ownership boundaries across mission/session/memory.
- [x] T2 Plan: break the architecture into implementation slices covering
      supervisor state, outer-loop execution, verifier adapters, memory/learning
      writeback, and operator surfaces.
- [x] T3 Align: map existing Tau subsystems (`tau-session`, `tau-memory`,
      cortex, `tau-orchestrator`, gateway/TUI) into the loop and identify
      compatibility/migration boundaries.

Architecture handoff:
- Contract source: `specs/3654/spec.md`.
- Implementation approach: `specs/3654/plan.md`.
- State owner: the mission id links gateway mission state, session lineage,
  action-history learning, verifier records, and operator surfaces.
- Product direction: subsequent work must turn this from deterministic mission
  proof into a live autonomous coding harness that can run against a real
  workspace.

## Implementation slice: mission completion outcome snapshots

- [x] T4 RED: add a gateway stream contract for mission completion outcome
      snapshots, covering `complete_task(status="partial")` as a
      `mission.checkpointed` operator event and `complete_task(status="blocked")`
      as a blocked operator snapshot.
- [x] T5 GREEN: thread completion-signal outcomes into the streamed
      `response.operator_turn_state.snapshot` payload without removing legacy
      `response.completed` compatibility.
- [x] T6 CLOSEOUT: document and publish evidence that the first governed Ralph
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
- [x] T9 CLOSEOUT: verify the handoff with the mission outcome snapshot guard,
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

## Implementation slice: verifier-blocked recovery learning bulletin replay

- [x] T14 Specify: require `gateway_verifier` action-history records from
      fail-closed verifier blocks to appear in later Ralph learning bulletins with
      reason-code evidence.
- [x] T15 RED: add gateway coverage proving a follow-up request after a verifier-blocked
      fabricated-progress mission receives a `## Learning Insights` bulletin containing
      `gateway_verifier` and the exhausted verifier `reason_code`.
- [x] T16 GREEN: preserve verifier reason-code evidence through action-history failure
      pattern rendering so the next Ralph-loop prompt can avoid repeating assistant-only
      or read-only-only completion claims.
- [x] T17 CLOSEOUT: verify the replay path alongside verifier-blocked persistence,
      existing learning bulletin behavior, formatting, clippy, and Cargo manifest stability.

Learning bulletin replay boundary:
- `gateway_verifier` is the learning signal name for verifier-blocked recovery evidence;
      it must not become a normal observed tool row.
- `reason_code` is part of the replay contract, not only mission-state metadata; later
      Ralph-loop prompts need the exhausted fail-closed reason to steer recovery.
- `complete_task` remains reserved for explicit mission completion signals, not verifier
      block replay evidence.

Evidence:
- `cargo test -p tau-gateway verifier_blocked_learning_bulletin -- --test-threads=1`
      proves `gateway_verifier` and `claimed_completion_without_tool_evidence_exhausted`
      survive from action history into a follow-up `## Learning Insights` bulletin.
- `cargo test -p tau-gateway verifier_blocked_learning -- --test-threads=1`
      keeps verifier-blocked persistence and `complete_task` separation intact.
- `cargo test -p tau-gateway regression_openresponses_injects_learning_insights_into_followup_system_prompt -- --test-threads=1`
      keeps existing generic learning-bulletin injection behavior intact.
- `cargo fmt --check`, `cargo clippy -p tau-gateway --tests --no-deps -- -D warnings`,
      and `git diff --quiet -- Cargo.toml` passed for the replay slice.

## Product implementation track: live autonomous coding harness

The next slices convert the Ralph supervisor from mission proof into a real
coding-agent harness. These tasks intentionally target product behavior: Tau must
own a coding mission, mutate a workspace, run verifiers, persist progress,
resume after interruption, and stop honestly when blocked.

### Slice A: Coding mission contract and state

- [x] T18 RED: add failing `tau-agent-core` coverage for a
      `CodingMissionRunner` contract that accepts repo path, issue or goal,
      base branch, branch prefix, verifier commands, PR mode, and allowed roots.
- [x] T19 GREEN: add `CodingMissionConfig`, `CodingMissionPhase`,
      `CodingMissionState`, and `CodingMissionEvent` with atomic JSON
      persistence under the existing mission state root.
- [x] T20 GREEN: link coding mission state to `MissionSnapshot` by mission id,
      session key, verifier records, artifact refs, and learning records rather
      than creating a disconnected harness state file.
- [x] T21 VERIFY: run focused state serialization, schema-version, and
      corrupted-state fail-closed tests.

Done when:
- A coding mission can be created, loaded, and inspected without a live model.
- Persisted state shows `intake`, `planned`, `blocked`, and `completed` phases
  as typed states, not only text summaries.
- Invalid or out-of-root repo paths fail before any mutation.

Evidence:
- RED: `CARGO_TARGET_DIR=/tmp/rust_pi-3654-coding-mission-target cargo test -p tau-agent-core coding_mission -- --test-threads=1` failed before implementation because `CodingMissionConfig`, `CodingMissionState`, `CodingMissionPhase`, persistence helpers, and `CodingMissionError` did not exist.
- GREEN/VERIFY: `CARGO_TARGET_DIR=/tmp/rust_pi-3654-coding-mission-target cargo test -p tau-agent-core coding_mission -- --test-threads=1` passed 5 tests covering typed state persistence/load, `MissionSnapshot` projection with artifact, verifier, and learning-record linkage, out-of-root rejection, corrupt JSON fail-closed behavior, and unsupported schema-version rejection.
- REGRESSION: `CARGO_TARGET_DIR=/tmp/rust_pi-3654-coding-mission-target cargo test -p tau-agent-core` passed 228 unit tests, 2 mission harness tests, and 9 doc tests.
- QUALITY: `cargo fmt --check`, `git diff --check`, `scripts/dev/roadmap-status-sync.sh --check --quiet`, and `CARGO_TARGET_DIR=/tmp/rust_pi-3654-coding-mission-target cargo clippy -p tau-agent-core -- -D warnings` passed.

### Slice B: Workspace executor and safety policy

- [ ] T22 RED: add failing coverage for a workspace executor that records cwd,
      argv, stdout/stderr paths, exit status, elapsed time, and reason code for
      every command.
- [ ] T23 GREEN: implement the executor on top of existing Tau tool/background
      job primitives so long-running verifier commands are supervised and
      recoverable.
- [ ] T24 GREEN: enforce command policy for allowed roots, denied destructive
      commands, optional network use, and explicit mutation phases.
- [ ] T25 VERIFY: prove denied commands leave no repo mutation and successful
      commands produce durable command evidence.

Done when:
- The runner can execute `git status`, `git diff`, and one verifier command in
  a target repo with durable artifacts.
- `rm -rf`, force push, out-of-root writes, and missing cwd fail closed with
  operator-visible reason codes.

### Slice C: Git lifecycle

- [ ] T26 RED: add failing tests for branch creation, branch reuse, dirty-tree
      detection, and base-branch mismatch handling.
- [ ] T27 GREEN: implement `prepare_branch` for checkout/fetch-free local
      operation first, with explicit remote fetch support only as an opt-in.
- [ ] T28 RED: add failing tests requiring completion to include a non-empty
      git diff or commit linked to the mission id.
- [ ] T29 GREEN: implement commit packaging with the repo Lore trailer contract
      when the target repo requires it, and a generic mission-linked commit
      message otherwise.
- [ ] T30 VERIFY: run the git lifecycle against a disposable fixture repo.

Done when:
- Tau can create a branch, make a mission-linked commit, and report exact commit
  hash and changed files.
- Dirty pre-existing user changes are detected and either preserved or block the
  mission before mutation.

### Slice D: Real outer loop

- [ ] T31 RED: add failing coverage where the first verifier run fails, the
      runner records RED evidence, and the mission remains `executing` rather
      than `completed`.
- [ ] T32 GREEN: implement the outer loop:
      `intake -> plan -> prepare_branch -> act -> verify -> replan/continue ->
      commit/pr_ready/block/complete`.
- [ ] T33 GREEN: require verifier pass plus mutation evidence before completion;
      assistant text alone cannot complete a coding mission.
- [ ] T34 RED: add failing coverage for impossible verifier commands and policy
      denials producing `blocked` missions with exact evidence.
- [ ] T35 VERIFY: run the loop against a fixture repo with one intentionally
      failing test that becomes green after a controlled edit.

Done when:
- A mission can pass through at least two iterations without manual steering.
- Failing verifiers become the next iteration input.
- Completion is impossible without command evidence, verifier pass, and git
  mutation/commit evidence.

### Slice E: Crash/resume

- [ ] T36 RED: add a crash fixture that kills the runner after RED evidence is
      persisted and before the edit phase finishes.
- [ ] T37 GREEN: implement `mission resume <mission-id>` for coding missions,
      restoring repo, branch, phase, pending verifier, and latest learning
      context from durable state.
- [ ] T38 RED: add a crash fixture that kills the runner after edits but before
      commit.
- [ ] T39 GREEN: resume from edited working tree without discarding user or
      runner changes; block if the diff no longer matches the saved mutation
      fingerprint.
- [ ] T40 VERIFY: run both crash/resume fixtures against the disposable repo.

Done when:
- Killing the Tau process mid-mission does not lose phase state, command
  evidence, verifier output, branch name, or pending next action.
- Resume never starts over silently.

### Slice F: PR-ready and GitHub path

- [ ] T41 RED: add failing coverage for `pr_ready` output when GitHub auth is
      absent: exact branch, commit, title, body, and manual `gh pr create`
      command are recorded.
- [ ] T42 GREEN: implement PR-ready bundle generation from mission state,
      verifier evidence, changed files, and risk/rollback notes.
- [ ] T43 RED: add failing coverage for optional `gh pr create --draft` when
      GitHub auth is present in the environment.
- [ ] T44 GREEN: implement opt-in draft PR creation with URL capture and
      mission-state update.
- [ ] T45 VERIFY: test missing-auth, dry-run, and live-credential-gated paths
      separately.

Done when:
- Missing credentials block only the PR creation step, not the coding mission.
- With credentials, Tau can create a draft PR and persist the PR URL.

### Slice G: Operator command center integration

- [ ] T46 RED: extend `tau-unified status` tests to require active coding
      mission id, phase, repo, branch, verifier, last failure, and PR-ready or
      PR URL state.
- [ ] T47 GREEN: surface coding mission state through the existing
      control-plane snapshot and gateway mission endpoints.
- [ ] T48 RED: extend TUI mission commands to show coding mission phase,
      verifier result, changed files, and resume command.
- [ ] T49 GREEN: wire `/missions`, `/mission <id>`, and `/resume <id>` to the
      real coding mission runner state.
- [ ] T50 VERIFY: run `scripts/run/test-tau-unified.sh status_contract` plus
      focused TUI mission tests.

Done when:
- Operators can see what Tau is doing without opening raw state files.
- The TUI can resume a blocked or checkpointed coding mission through the real
  runner, not a simulated harness.

### Slice H: Live end-to-end benchmark

- [ ] T51 RED: add `scripts/dev/test-full-autonomous-coding-loop.sh` with a
      disposable repo fixture and fail-closed assertions for all required
      artifacts.
- [ ] T52 GREEN: wire the M334
      `repo_spec_to_pr_feature_delivery` benchmark task to the live coding
      mission runner.
- [ ] T53 VERIFY: run the full local loop without PR creation:
      branch, RED verifier, edit, GREEN verifier, commit, pr-ready bundle.
- [ ] T54 VERIFY: run the full loop with one forced crash/resume injection.
- [ ] T55 VERIFY: run the blocked-task fixture and prove Tau stops honestly.
- [ ] T56 CLOSEOUT: update `README.md`, `docs/guides/canonical-product-proof.md`,
      and M334 docs to distinguish the live harness from older deterministic
      proof simulation.

Done when:
- Tau, not Codex, completes the fixture repo task end-to-end.
- The benchmark records no routine human steering beyond provider auth or one
  major direction choice.
- The final artifact includes mission id, branch, commit, verifier transcript,
  resume evidence, and PR-ready or PR URL state.

## Task ordering

1. Finish Slice A first; no later slice may create a parallel mission state
   format.
2. Finish Slice B before any model-driven edits; command policy and durable
   evidence must exist before the harness can mutate real repos.
3. Finish Slice C before PR work; branch and commit behavior must be boring.
4. Finish Slice D before claiming autonomy; this is where Tau stops being a
   single-turn caller.
5. Finish Slice E before marketing this as durable.
6. Finish Slice F only after local repo operation is reliable.
7. Finish Slice G once state is real enough to inspect.
8. Finish Slice H last; it is the product acceptance run, not a substitute for
   the implementation slices.
