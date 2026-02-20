# Spec: Issue #2717 - Implement Cortex runtime heartbeat loop, cross-session memory bulletin, and ArcSwap prompt injection

Status: Implemented

## Problem Statement
`tasks/spacebot-comparison.md` still has unchecked G3 items after M111/M112: there is no runtime Cortex loop that periodically reads memory across sessions, produces a bulletin, and injects that bulletin into new-session system prompts. Current gateway sessions always initialize from a static startup prompt.

## Acceptance Criteria

### AC-1 Cortex runtime core exists with deterministic bulletin state
Given gateway runtime startup,
When Cortex runtime is initialized,
Then a first-class `Cortex` runtime struct is available with an `ArcSwap<String>` bulletin snapshot and deterministic refresh diagnostics.

### AC-2 Cortex refresh reads cross-session memory on heartbeat cadence
Given memory records exist across multiple session memory stores under gateway state,
When Cortex refresh executes on heartbeat cadence,
Then it scans session memory stores deterministically and builds a bounded cross-session bulletin input.

### AC-3 Cortex bulletin refresh supports LLM summarization with safe fallback
Given gathered cross-session memory input,
When Cortex refresh runs,
Then it attempts LLM summarization and stores the resulting bulletin snapshot; if LLM output fails or is empty, deterministic fallback bulletin text is stored instead.

### AC-4 New-session prompt initialization injects latest Cortex bulletin via ArcSwap
Given a non-empty Cortex bulletin snapshot,
When a new gateway session is initialized,
Then the initialized system prompt includes the latest bulletin snapshot from `ArcSwap<String>`.

### AC-5 Existing auth/session behavior remains stable
Given existing gateway auth/session contracts,
When Cortex bulletin injection is introduced,
Then unauthorized paths and existing session continuation behavior remain unchanged.

### AC-6 Scoped verification gates pass
Given this implementation slice,
When scoped checks run,
Then `cargo fmt --check`, `cargo clippy -p tau-agent-core -p tau-gateway -- -D warnings`, and targeted conformance/regression tests pass.

## Scope

### In Scope
- Add a `Cortex` runtime primitive in `tau-agent-core` with:
  - heartbeat-refreshable bulletin generation flow
  - cross-session memory-store scanning
  - `ArcSwap<String>` bulletin snapshot storage/access
- Start/stop Cortex refresh loop in gateway server using runtime heartbeat interval configuration.
- Inject bulletin snapshot into new-session system prompt initialization paths.
- Add conformance/regression tests for refresh, fallback, and injection behavior.
- Update `tasks/spacebot-comparison.md` G3 checklist items completed by this slice.

### Out of Scope
- Full five-process runtime orchestration (G1 complete architecture conversion).
- Cortex admin chat model/tooling expansion beyond existing endpoints.
- Dashboard/UI surfaces for viewing bulletin details.

## Conformance Cases
- C-01 (integration): Cortex refresh scans multiple session memory-store roots and aggregates bounded records deterministically.
- C-02 (integration): successful LLM completion updates stored bulletin snapshot.
- C-03 (regression): LLM error/empty output path stores deterministic fallback bulletin snapshot.
- C-04 (integration): new gateway session initialization uses bulletin-injected system prompt text.
- C-05 (regression): existing session continuation does not rewrite leading system message with duplicate bulletin content.
- C-06 (verify): scoped fmt/clippy/targeted tests pass.

## Success Metrics / Observable Signals
- Cortex bulletin snapshot changes over time without gateway restart when memory changes.
- New sessions receive bulletin context automatically through existing prompt initialization flow.
- Gateway scoped test suite remains green with deterministic coverage for fallback and prompt injection.
