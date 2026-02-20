# Milestone M130 - Tau Ops Dashboard PRD Phase 1B (Auth + Protected Routing)

Status: InProgress

## Scope
Implement PRD phase-1B auth/protected-route scaffolding for Tau Ops Dashboard:
- add gateway auth bootstrap contract endpoint for ops UI initialization,
- extend `tau-dashboard-ui` SSR shell with auth-mode and login/protected markers,
- integrate `/ops` and `/ops/login` gateway routes backed by auth-aware shell context.

## Linked Issues
- Epic: #2784
- Story: #2785
- Task: #2786

## Success Signals
- `/gateway/auth/bootstrap` reports deterministic auth mode metadata without secret leakage.
- `/ops` and `/ops/login` return auth-aware shell HTML with route/auth markers.
- Existing dashboard shell and password session issuance contracts remain stable.
