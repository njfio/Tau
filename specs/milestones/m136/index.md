# Milestone M136 - Tau Ops Dashboard PRD Phase 1H (Command Center Control Contracts)

Status: InProgress

## Scope
Implement command-center control-mode/action SSR contracts in Tau Ops shell:
- control mode + rollout gate markers bound to dashboard control/health snapshot,
- action affordance markers for pause/resume/refresh bound to allowed actions and active mode,
- last-action metadata markers for operator audit visibility.

## Linked Issues
- Epic: #2808
- Story: #2809
- Task: #2810

## Success Signals
- `/ops` shell exposes deterministic control markers derived from live dashboard snapshot.
- Action-marker contracts are deterministic and testable in SSR output.
- Existing phase-1A..1G contracts remain green.
