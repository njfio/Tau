# Spec: Issue #3055 - Integration breadth expansion for channel scope and ordered tool flows

Status: Reviewed

## Problem Statement
The integration suite validates a basic memory roundtrip but does not explicitly verify ordered multi-step tool payload behavior within a single prompt or channel-scope filtering across same-workspace memories. This leaves regression risk for routing semantics and scenario composability.

## Acceptance Criteria

### AC-1 Ordered multi-step tool payload visibility
Given an agent prompt that triggers multiple `memory_search` tool calls in one execution,
When integration assertions inspect tool payloads,
Then they can assert ordered payload results deterministically.

### AC-2 Channel-scope routing behavior is verified in integration suite
Given same-workspace memories in different channels,
When searching with a channel-scoped request,
Then results include only matching channel memories.

### AC-3 Verification gates remain green
Given integration test updates,
When running validation,
Then targeted integration tests and baseline fmt/clippy/check gates pass.

## Scope

### In Scope
- `tests/integration/tests/agent_tool_memory_roundtrip.rs`
- `specs/milestones/m192/index.md`
- `specs/3055/*`

### Out of Scope
- Memory runtime ranking algorithm changes.
- New production API endpoints.

## Conformance Cases
- C-01 (AC-1): multi-step prompt produces ordered `memory_search` payload assertions (scoped miss then scoped hit).
- C-02 (AC-2): channel-scoped search excludes same-workspace memories from other channels.
- C-03 (AC-3): integration validation command set passes.

## Success Metrics / Observable Signals
- `cargo test -p integration-tests integration_spec_3055_c01_agent_multi_step_search_preserves_ordered_payloads -- --nocapture`
- `cargo test -p integration-tests integration_spec_3055_c02_channel_scope_filters_same_workspace_records -- --nocapture`
- `cargo test -p integration-tests -- --nocapture --test-threads=1`
- `cargo fmt --check`
- `cargo clippy -p integration-tests -- -D warnings`
- `cargo check -q`

## Approval Gate
P1 scope: spec authored/reviewed by agent; implementation proceeds and is flagged for human review in PR.
