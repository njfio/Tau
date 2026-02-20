# Plan: Issue #2717 - Implement Cortex runtime heartbeat loop, cross-session memory bulletin, and ArcSwap prompt injection

## Approach
1. Add RED conformance/regression tests for Cortex refresh aggregation, LLM/fallback bulletin behavior, and gateway prompt injection.
2. Implement `tau-agent-core` Cortex runtime module with:
   - deterministic cross-session memory scanning
   - bounded bulletin input preparation
   - LLM summarize + deterministic fallback
   - `ArcSwap<String>` bulletin snapshot.
3. Integrate Cortex loop into gateway server lifecycle with heartbeat cadence and clean shutdown.
4. Wire gateway new-session initialization to compose base system prompt + latest bulletin snapshot from ArcSwap.
5. Run scoped verification commands and update G3 checklist state.

## Affected Modules
- `crates/tau-agent-core/src/lib.rs`
- `crates/tau-agent-core/src/cortex_runtime.rs` (new)
- `crates/tau-agent-core/Cargo.toml`
- `crates/tau-gateway/src/gateway_openresponses.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`
- `tasks/spacebot-comparison.md`

## Risks / Mitigations
- Risk: periodic refresh loop introduces shutdown leaks or hangs.
  - Mitigation: explicit shutdown channel + awaited task join in server shutdown path.
- Risk: bulletin injection could alter existing-session prompts unexpectedly.
  - Mitigation: inject only through session initialization path; regression test continuation behavior.
- Risk: memory scan over many sessions becomes unbounded.
  - Mitigation: deterministic per-session/global caps and stable sort/truncation.

## Interfaces / Contracts
- New `tau-agent-core` `Cortex` runtime API (public):
  - initialize with memory-store root + limits
  - refresh bulletin async via LLM client/model
  - expose bulletin ArcSwap snapshot + composed system prompt helper
- Gateway session initialization contract remains unchanged except system prompt content now may include bulletin suffix for newly initialized sessions.

## ADR
- Not required. No new dependency family or wire protocol change; `arc-swap` is already a workspace dependency and only extended into existing crates.
