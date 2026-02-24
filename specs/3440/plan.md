# Plan: Issue #3440 - Planning docs status reconciliation

1. Capture RED by showing stale phrases in both target docs that conflict with implemented status.
2. Update `docs/planning/integration-gap-closure-plan.md`:
   - reframe from "gap baseline" to "closure status + expansion track".
   - clarify each area (RL/dashboard/auth/TUI) with current integrated baseline + remaining scope.
3. Update `docs/planning/true-rl-roadmap-skeleton.md`:
   - replace future-only boundary statement with current integrated baseline statement.
   - keep stage structure while clarifying stages are hardening/expansion.
4. Run docs quality checks and roadmap sync checks.
5. Record GREEN evidence in tasks/spec.

## Affected Modules
- `docs/planning/integration-gap-closure-plan.md`
- `docs/planning/true-rl-roadmap-skeleton.md`
- `specs/3440/*`

## Risks / Mitigations
- Risk: over-claiming completion.
  - Mitigation: keep explicit "remaining expansion" notes for each capability.
- Risk: drift with roadmap-linked issue references.
  - Mitigation: preserve existing stage links and present them as long-horizon hardening track.
