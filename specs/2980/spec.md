# Spec: Issue #2980 - Split OpenAI compatibility handlers into openai_compat_runtime module

Status: Reviewed

## Problem Statement
`crates/tau-gateway/src/gateway_openresponses.rs` still contains OpenAI compatibility handler logic inline, increasing hotspot complexity and making routing/runtime behavior harder to reason about.

## Acceptance Criteria

### AC-1 OpenAI compatibility handlers are extracted to a dedicated runtime module
Given the gateway OpenResponses source tree,
When inspecting OpenAI compatibility endpoint implementation,
Then handler functions for `/v1/chat/completions`, `/v1/completions`, and `/v1/models` live in `gateway_openresponses/openai_compat_runtime.rs`.

### AC-2 Route behavior and telemetry semantics remain unchanged
Given existing OpenAI compatibility routes,
When requests are sent to OpenAI compatibility endpoints,
Then auth/rate-limit enforcement, ignored-field telemetry, reason codes, and response semantics remain unchanged.

### AC-3 OpenAI compatibility tests remain green
Given the gateway test suite,
When running targeted OpenAI compatibility tests,
Then existing compatibility conformance/integration tests pass.

### AC-4 Hotspot size is reduced again
Given baseline `gateway_openresponses.rs` line count,
When extraction is complete,
Then file line count decreases from baseline.

## Scope

### In Scope
- Extract OpenAI compatibility handlers into `openai_compat_runtime.rs`.
- Extract tightly-coupled streaming helper plumbing needed by moved handlers.
- Preserve route registration and endpoint constants.
- Run targeted regression/verification gates.

### Out of Scope
- API schema changes.
- auth policy changes.
- telemetry schema changes.

## Conformance Cases
- C-01: `openai_compat_runtime.rs` contains extracted OpenAI compatibility handlers.
- C-02: route table still binds OpenAI compatibility paths to correct handlers.
- C-03: targeted OpenAI compatibility tests pass.
- C-04: `gateway_openresponses.rs` line count is lower than baseline.

## Success Metrics / Observable Signals
- `cargo test -p tau-gateway openai_chat_completions -- --test-threads=1` passes.
- `cargo test -p tau-gateway openai_completions -- --test-threads=1` passes.
- `cargo test -p tau-gateway openai_models -- --test-threads=1` passes.
- `gateway_openresponses.rs` line count decreases versus baseline.

## Approval Gate
P1 scope: spec authored/reviewed by agent; implementation proceeds and is flagged for human review in PR.
