# Spec: Issue #2608 - Integration suite bootstrap under tests/integration

Status: Implemented

## Problem Statement
Tau currently lacks a repository-level integration suite entrypoint that validates cross-crate behavior from the agent loop through tool execution into memory persistence and retrieval. Without this bootstrap, end-to-end regressions can hide behind crate-local tests and there is no repeatable scaffold for additional cross-crate conformance tests.

## Acceptance Criteria

### AC-1 Workspace-integrated bootstrap package exists
Given the repository root workspace,
When `cargo test -p tau-integration-tests` is executed,
Then a dedicated `tests/integration` package builds and runs from the workspace without requiring external credentials or services.

### AC-2 End-to-end flow validates agent -> tools -> memory write -> memory search
Given an agent configured with `memory_write` and `memory_search` tools,
When the mocked model requests a write followed by a search in one deterministic prompt flow,
Then tool execution succeeds and the search result includes the memory written in the same flow.

### AC-3 Harness is deterministic and isolated
Given repeated local executions,
When the integration test runs,
Then it uses an isolated temp memory state directory, avoids network/provider calls, and leaves no shared mutable state dependency between runs.

### AC-4 Pattern is reusable for future integration slices
Given future cross-crate scenarios,
When engineers add tests under `tests/integration`,
Then they can reuse the same mocked-client + tool-registration pattern without coupling to crate-internal test helpers.

## Scope

### In Scope
- Add workspace member package at `tests/integration` for repository-level integration tests.
- Add initial conformance test covering AC-2 with real `tau-agent-core` + `tau-tools` integration.
- Use deterministic in-test mock LLM client queueing tool-call and final assistant responses.

### Out of Scope
- Live provider HTTP integration.
- Multi-process distributed orchestration.
- Additional cross-service transports (Slack/GitHub/websocket) beyond this bootstrap slice.

## Conformance Cases
- C-01 (AC-1, integration): `integration_spec_2608_c01_workspace_runs_integration_package`
- C-02 (AC-2, conformance): `conformance_spec_2608_c02_agent_tool_memory_roundtrip`
- C-03 (AC-3, regression): `regression_spec_2608_c03_harness_uses_isolated_memory_state`
- C-04 (AC-4, functional): `functional_spec_2608_c04_pattern_is_composable_for_new_scenarios`

## Success Metrics / Observable Signals
- `cargo test -p tau-integration-tests` passes with C-01..C-04.
- The primary integration test asserts non-empty `memory_search.matches` containing the newly written memory id/summary.
- `cargo fmt --check` and scoped `clippy` are green for touched workspace members.
