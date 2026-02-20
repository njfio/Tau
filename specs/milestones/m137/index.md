# Milestone M137 - Tau Ops Dashboard PRD Phase 1I (Command Center Timeline Range Contracts)

Status: InProgress

## Scope
Implement command-center timeline chart SSR contracts in Tau Ops shell:
- timeline chart marker block with live queue timeline point metadata,
- query-driven 1h/6h/24h range selector markers,
- deterministic range links preserving route theme/sidebar shell state.

## Linked Issues
- Epic: #2812
- Story: #2813
- Task: #2814

## Success Signals
- `/ops` shell exposes timeline chart markers backed by dashboard snapshot timeline data.
- `/ops` query range state (`1h|6h|24h`) is reflected by deterministic selector markers and safe fallback defaults.
- Existing phase-1A..1H contracts remain green.
