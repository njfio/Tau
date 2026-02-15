# Doc Density Allocation Plan

This guide defines crate-level marker quotas and ownership cadence for M23.

Primary allocation artifact:
- `tasks/policies/m23-doc-allocation-plan.json`

Operator-readable summary:
- `tasks/reports/m23-doc-allocation-plan.md`

## Goal

Bridge from baseline marker volume (`1,438`) to M23 gate floor (`>=3,000`)
using explicit crate-level quotas, owner domains, and checkpoint dates.

## Contract Fields

`m23-doc-allocation-plan.json` includes:

- baseline/current/target totals
- checkpoint schedule (`m23-cp1`/`m23-cp2`/`m23-cp3`)
- owner-domain review cadence
- crate quota allocations (`current`, `target`, `delta_required`)
- missed-target escalation workflow

## Owner Domains

- `runtime-core`: weekly review (`7` days)
- `tools-cli`: `10`-day cadence
- `transport-bridges`: `10`-day cadence
- `gateway-provider`: `10`-day cadence
- `operations`: `14`-day cadence
- `training-runtime`: `14`-day cadence

## Checkpoint Rhythm

- `m23-cp1` (`2026-03-31`): min `2,000` markers
- `m23-cp2` (`2026-04-30`): min `2,450` markers
- `m23-cp3` (`2026-05-30`): min `3,000` markers

Gate issue for final signoff:
- `#1701`

## Escalation

If checkpoint targets are missed:

1. Domain owner files remediation plan within 72h and links `#1656`.
2. Repeated or >5% misses are escalated as blockers on `#1701`.
3. Multi-domain misses require tracker escalation on `#1678`.
