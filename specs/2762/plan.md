# Plan: Issue #2762 - Discord placeholder message + progressive edit streaming (G10)

## Approach
1. Add RED integration tests in `multi_channel_outbound` for Discord provider placeholder + PATCH behavior.
2. Implement a provider-mode Discord progressive edit delivery path in outbound dispatcher:
   - POST placeholder
   - extract message id
   - PATCH accumulated content per chunk
3. Keep fallback to existing chunked POST path for >2000-char responses.
4. Run scoped gates + local live validation against a mock Discord API.
5. Update checklist evidence in `tasks/spacebot-comparison.md`.

## Affected Modules
- `crates/tau-multi-channel/src/multi_channel_outbound.rs`
- `tasks/spacebot-comparison.md`

## Risks / Mitigations
- Risk: missing provider message id from placeholder response breaks edit flow.
  - Mitigation: fail fast with structured outbound error and test expected response shape.
- Risk: accidental behavior drift for long Discord responses.
  - Mitigation: add regression test for >2000-char fallback chunked POST behavior.

## Interfaces / Contracts
- Internal outbound behavior contract update:
  - Discord provider delivery (<=2000 chars) uses placeholder+PATCH progressive edits.
  - Discord provider delivery (>2000 chars) continues chunked POST fallback.
- No CLI flag/dependency changes.

## ADR
- Not required: no dependency/protocol architecture change.
