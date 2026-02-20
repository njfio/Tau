# Milestone M149 - Tau Ops Dashboard PRD Phase 1U (Chat Token Counter Contracts)

Status: InProgress

## Scope
Implement deterministic chat token-counter marker contracts:
- explicit active-session token counter marker attributes on the chat panel,
- deterministic token counter values derived from active-session usage snapshot fields,
- route-safe behavior across `/ops`, `/ops/chat`, and `/ops/sessions` while preserving existing contracts.

## Linked Issues
- Epic: #2860
- Story: #2861
- Task: #2862

## Success Signals
- Chat panel exposes deterministic token-counter marker attributes.
- Marker values mirror active-session usage summary fields.
- Existing chat/session/command-center visibility and detail contracts remain green.
