# Spec: Issue #3570 - Memory Distill and Cortex Visibility Integration Across TUI/Webchat

Issue: https://github.com/njfio/Tau/issues/3570

## Objective
Make memory distill behavior observable and trustworthy end-to-end by:
- broadening deterministic distill extraction coverage,
- surfacing first-class distill runtime state in gateway status and web surfaces,
- exposing a dedicated TUI command for distill status,
- proving the integrated flow with real-path tests.

## Inputs/Outputs
### Inputs
- Session JSONL events in `.tau/gateway/openresponses/sessions/*.jsonl`.
- User messages appended to session streams.
- Existing memory state directory and memory APIs.
- Gateway status aggregation and web UI status payloads.
- TUI sync payload from `/gateway/status`.

### Outputs
- Distill runtime extracts additional user fact/goal/preference/constraint candidates.
- Distill runtime writes memory entries (idempotent semantics) with explicit reason codes.
- Gateway status payload includes richer memory-distill fields (including recent writes + last cycle stats).
- TUI supports `/memory-distill` command (with `/memory` retained as alias) and renders distill snapshot.
- Webchat/dashboard status path exposes the same distill runtime contract for parity.

## Boundaries/Non-goals
- No full TUI layout redesign.
- No provider model/tool orchestration redesign.
- No new memory ontology framework.
- No silent fallback behavior; explicit errors/reason codes only.

## Failure Modes
1. Session directory unreadable.
2. Session file parse failure or malformed JSONL.
3. Checkpoint load/save failure.
4. Memory write failure.
5. Distill extraction yields no candidates.
6. Status/web payload missing distill fields.
7. TUI command path receives missing/partial distill payload.

## Acceptance Criteria (Testable Booleans)
1. Distill extraction coverage
- [ ] For a deterministic set of user utterances (identity/fact/preference/goal/constraint), candidate extraction returns expected normalized memory candidates.

2. Distill write + observability
- [ ] Distill cycle writes memory entries for new candidates and reports explicit per-cycle counters and reason codes.
- [ ] Distill cycle remains idempotent for already-processed source entries.

3. Status/web contract
- [ ] `/gateway/status` includes `gateway.web_ui.memory_distill_runtime` with required keys:
  `enabled`, `in_flight`, `cycle_count`, `writes_applied`, `write_failures`,
  `checkpoint_path`, `last_reason_codes`, `last_cycle_*`, and `recent_writes`.

4. TUI command contract
- [ ] `/memory-distill` command is recognized and renders current distill snapshot.
- [ ] `/memory` remains functional as an alias.

5. Webchat/dashboard parity
- [ ] Webchat/dashboard status consumers can render the same distill contract without missing-field regressions.

6. Integrated flow
- [ ] Integration test proves: append user event -> run distill cycle -> memory persisted -> status reflects write -> TUI-visible snapshot includes updated counters/recent write.

7. Failed-turn operator visibility
- [ ] If runtime emits failed-turn status with error and no assistant answer text, TUI appends a concise explicit failure line to Assistant pane (not only Timeline/Turn).

## Files To Touch
- `crates/tau-gateway/src/gateway_openresponses/memory_distill_runtime.rs`
- `crates/tau-gateway/src/gateway_openresponses/status_runtime.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`
- `crates/tau-gateway/src/gateway_openresponses/webchat_page.rs` (if status surface mapping changes required)
- `crates/tau-tui/src/main.rs`
- `crates/tau-tui/tests/tui_demo_smoke.rs` (only if command/help coverage belongs here)

## Error Semantics
- Distill/runtime internals return typed errors or deterministic reason codes; no silent swallow.
- Entrypoints/status surfaces expose failures via explicit fields/reason codes.
- TUI render path degrades to explicit `(missing)` markers for absent fields but does not fabricate success.

## Test Plan
### RED (new failing tests first)
- Distill candidate extraction unit tests for expanded phrase coverage.
- Distill cycle integration test asserting recent writes + last-cycle counters.
- Gateway status/web-ui contract test asserting enriched `memory_distill_runtime` fields.
- TUI parser/command test for `/memory-distill` and alias behavior.
- TUI parser/render test for failed-turn event path with assistant-pane error fallback.

### GREEN
- Implement minimal runtime/status/TUI changes to satisfy tests.

### REFACTOR
- Deduplicate distill status formatting and reason-code handling.
- Keep functions concise and deterministic.

### INTEGRATION
- Run targeted gateway + TUI tests covering full flow.
- Smoke-check status payload shape from running gateway fixture.

## Implementation Status
Status: Implemented

### Verification Evidence
- `cargo test -p tau-gateway unit_distill_candidates_extracts_alias_location_and_allergy_constraint -- --nocapture`
- `cargo test -p tau-gateway integration_memory_distill_cycle_writes_memory_and_checkpoints_processed_entries -- --nocapture`
- `cargo test -p tau-gateway integration_gateway_status_endpoint_returns_service_snapshot -- --nocapture`
- `cargo test -p tau-tui unit_parse_local_tui_command_maps_dashboard_tools_routines_cortex_memory_sync_and_colors -- --nocapture`
- `cargo test -p tau-tui integration_gateway_sync_snapshot_full_mode_reflects_status_contract_for_tools_and_cortex -- --nocapture`
- `cargo test -p tau-tui functional_spec_c15_agent_launch_summary_is_compact_and_command_oriented -- --nocapture`
