# Spec: Force Concrete Mutating Tool Choice On Ralph-Loop Recovery Retries

Status: Reviewed

## Problem Statement

Tau's Ralph-loop gateway now cuts off read-only exploration and retries mutation work with a larger timeout budget. Live `gpt-5.3-codex` runs still stall on the recovery turn because the gateway only forces a generic required-tool response. The provider returns no recovery tool call before timeout even when the retry prompt already requires mutation and forbids more broad reads.

## Scope

In scope:
- selecting a concrete mutating tool choice for recovery retries when the task clearly implies file creation or file mutation
- preferring `write` for create/new-folder/build/scaffold/generate style requests when `write` is available
- preserving fallback to generic `Required` when no safe concrete choice is inferred
- updating mutation-retry regressions to assert the chosen retry tool choice

Out of scope:
- changing first-attempt tool behavior
- redesigning provider prompt contracts beyond using the existing `ToolChoice::Tool` path
- non-mutation retries and validation-only retries

## Acceptance Criteria

### AC-1
Given a mutation-recovery retry for a create/build/scaffold/new-folder style task
When the gateway prepares the next retry request and `write` is available
Then it uses `ToolChoice::Tool { name: "write" }` instead of generic `Required`.

### AC-2
Given a mutation-recovery retry where no safe concrete mutating tool can be inferred
When the gateway prepares the retry request
Then it falls back to `ToolChoice::Required`.

### AC-3
Given existing gateway mutation-retry flows
When the updated concrete tool choice logic is applied
Then scoped gateway regressions continue to pass with updated expectations.

## Conformance Cases

- C-01: create task mutation retry with available `write` tool requests `ToolChoice::Tool { name: "write" }`. Maps to AC-1. Tier: Conformance.
- C-02: mutation retry without an inferred concrete tool falls back to `ToolChoice::Required`. Maps to AC-2. Tier: Conformance.
- C-03: read-only saturation retry still completes via mutation after the concrete retry tool choice is applied. Maps to AC-1, AC-3. Tier: Integration.
- C-04: timeout-after-read-only retry still compacts context and retries into mutation with the concrete retry tool choice. Maps to AC-1, AC-3. Tier: Integration.

## Success Metrics

- Live mutation-recovery retries no longer stop at a generic tool-required timeout for create/new-folder Phaser tasks.
- The retry request artifact in tests clearly shows a concrete mutating tool choice where the heuristic applies.
