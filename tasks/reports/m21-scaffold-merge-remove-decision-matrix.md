# M21 Scaffold Merge/Remove Decision Matrix

- Generated: 2026-02-16T00:00:00Z
- Policy: `tasks/policies/scaffold-merge-remove-rubric.json`

## Summary

| Metric | Value |
| --- | ---: |
| Total candidates | 13 |
| Keep decisions | 8 |
| Merge decisions | 1 |
| Remove decisions | 4 |
| Unresolved decisions | 0 |

## Rubric

| Criterion | Weight | Direction |
| --- | ---: | --- |
| `operator_value` | 3 | higher better |
| `runtime_usage` | 3 | higher better |
| `maintenance_cost` | 2 | lower better |
| `test_posture` | 2 | higher better |

| Action | Threshold |
| --- | --- |
| remove | score <= 18 |
| keep | 19 <= score <= 35 |
| merge | score >= 36 |

## Decision Matrix

| Candidate | Action | Merge Target | Owner | Score | Rationale |
| --- | --- | --- | --- | ---: | --- |
| `tau-algorithm` | keep | - | `training-runtime` | 35 | Algorithm strategy layer is independently testable and should remain decoupled from runtime plumbing. |
| `tau-browser-automation` | remove | - | `tools-runtime` | 17 | Contract-only browser scaffolding remains low-usage and high-maintenance without live automation hooks. |
| `tau-contract-runner-remnants` | remove | - | `runtime-core` | 7 | No operator-facing value and no active runtime usage; dead remnant maintenance burden is unjustified. |
| `tau-custom-command` | keep | - | `events-runtime` | 25 | Custom command contracts provide operator value but are not yet high-frequency runtime paths. |
| `tau-dashboard-widget-contracts` | merge | tau-dashboard | `gateway-ui` | 37 | Widget contracts and runtime shell should converge into one maintained dashboard ownership boundary. |
| `tau-memory-postgres-backend` | remove | - | `memory-runtime` | 13 | Postgres path is scaffold-level and duplicates active memory ownership in retained runtime surfaces. |
| `tau-trainer` | keep | - | `training-runtime` | 35 | Top-level trainer lifecycle remains stable and easier to evolve as a focused crate boundary. |
| `tau-training-proxy` | keep | - | `training-runtime` | 25 | Proxy surface is optional and should remain isolated until live usage justifies deeper consolidation. |
| `tau-training-runner` | keep | - | `training-runtime` | 35 | Runner orchestration remains a clean runtime boundary with active test coverage. |
| `tau-training-store` | keep | - | `training-runtime` | 35 | Store ownership is explicit and reused by runner/tracer/trainer orchestration flows. |
| `tau-training-tracer` | keep | - | `training-runtime` | 33 | Tracer boundary is actively used and independently testable from runner/store concerns. |
| `tau-training-types` | keep | - | `training-runtime` | 35 | Shared training type boundary remains stable and keeps compile-time dependencies acyclic. |
| `tau-voice-runtime` | remove | - | `multi-channel-runtime` | 15 | Voice contracts are present but live speech pipeline is not yet in retained production flows. |
