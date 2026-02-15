# M23 Doc Allocation Plan

Generated at: 2026-02-15T17:41:19Z

## Summary

- Baseline total markers: `1438`
- Current total markers: `1486`
- Planned target total markers: `3000`
- M23 gate floor: `3000`

## Checkpoints

| Checkpoint | Date | Min Total Markers | Review Mode |
| --- | --- | ---: | --- |
| m23-cp1 | 2026-03-31 | 2000 | domain-owner review + delta validation |
| m23-cp2 | 2026-04-30 | 2450 | domain-owner review + quality spot audit sample |
| m23-cp3 | 2026-05-30 | 3000 | gate readiness review (#1701) |

## Quota Allocations

| Crate | Current | Target | Delta | Owner Domain | Cadence (days) |
| --- | ---: | ---: | ---: | --- | ---: |
| tau-agent-core | 283 | 363 | 80 | runtime-core | 7 |
| tau-cli | 36 | 96 | 60 | tools-cli | 10 |
| tau-coding-agent | 21 | 301 | 280 | runtime-core | 7 |
| tau-gateway | 37 | 127 | 90 | gateway-provider | 10 |
| tau-github-issues-runtime | 39 | 59 | 20 | transport-bridges | 10 |
| tau-memory | 48 | 118 | 70 | runtime-core | 7 |
| tau-multi-channel | 105 | 235 | 130 | transport-bridges | 10 |
| tau-onboarding | 66 | 186 | 120 | runtime-core | 7 |
| tau-ops | 20 | 104 | 84 | operations | 14 |
| tau-provider | 39 | 129 | 90 | gateway-provider | 10 |
| tau-runtime | 111 | 231 | 120 | runtime-core | 7 |
| tau-session | 41 | 111 | 70 | runtime-core | 7 |
| tau-skills | 43 | 113 | 70 | runtime-core | 7 |
| tau-startup | 22 | 92 | 70 | runtime-core | 7 |
| tau-tools | 45 | 165 | 120 | tools-cli | 10 |
| tau-training-types | 77 | 117 | 40 | training-runtime | 14 |

## Escalation

- If a checkpoint misses target by <=5%, domain owner submits remediation plan within 72h and tags issue #1656.
- If miss is >5% or repeats on consecutive checkpoints, open blocker under #1701 and escalate in tracker #1678.
- If two domains miss in same checkpoint, freeze non-critical doc PR lanes until recovery plan is approved.
