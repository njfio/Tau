# Milestone M148 - Tau Ops Dashboard PRD Phase 1T (Panel Visibility State Contracts)

Status: InProgress

## Scope
Implement deterministic chat/sessions panel visibility-state contracts:
- explicit `data-panel-visible` marker attributes on chat and sessions panels,
- deterministic route-based panel visibility states across `/ops`, `/ops/chat`, and `/ops/sessions`,
- preserve existing route marker and panel contract behavior.

## Linked Issues
- Epic: #2856
- Story: #2857
- Task: #2858

## Success Signals
- Chat and sessions panels expose `data-panel-visible` with deterministic true/false values.
- `/ops`, `/ops/chat`, and `/ops/sessions` render expected panel visibility-state combinations.
- Existing chat/sessions/command-center markers remain green.
