# Spec: Issue #3070 - ops memory-graph node size importance contracts

Status: Reviewed

## Problem Statement
`/ops/memory-graph` now exposes deterministic node/edge rows, but node-size
contracts are not surfaced. PRD checklist contract `2088` (node size reflects
importance) remains unverified for SSR rendering.

## Scope
In scope:
- Add deterministic node-size marker attributes for memory graph nodes.
- Derive size buckets and numeric size values from normalized importance.
- Validate size-marker rendering through UI and gateway conformance tests.

Out of scope:
- Node color contracts.
- Edge style and interactive graph behavior.
- New dependencies.

## Acceptance Criteria
### AC-1 `/ops/memory-graph` keeps deterministic graph contracts with explicit size marker surface
Given `/ops/memory-graph` renders,
when graph nodes are empty,
then deterministic graph panel/list markers remain present for size contract stability.

### AC-2 Node size markers reflect normalized importance
Given memory graph nodes exist with varying importance,
when `/ops/memory-graph` is rendered,
then each node exposes deterministic size bucket/value attributes derived from importance.

### AC-3 Non-memory-graph routes preserve hidden graph contracts
Given any non-memory-graph route renders,
when shell HTML is produced,
then graph panel markers remain present and hidden.

### AC-4 Existing memory-graph and memory explorer contracts remain green
Given existing memory graph/explorer specs,
when node-size contracts are added,
then selected conformance/regression suites remain green.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
|---|---|---|---|---|---|
| C-01 | AC-1 | Functional | `/ops/memory-graph` with no records | render route | panel/list markers remain deterministic with zero-count defaults |
| C-02 | AC-2 | Integration | graph nodes with low/high importance | render route | node rows include deterministic `data-node-size-bucket` and `data-node-size-px` |
| C-03 | AC-3 | Regression | route is not `/ops/memory-graph` | render route | graph panel markers remain present and hidden |
| C-04 | AC-4 | Regression | existing memory specs | rerun selected suites | previous contracts remain green |

## Success Metrics / Signals
- `cargo test -p tau-dashboard-ui spec_3070 -- --test-threads=1`
- `cargo test -p tau-gateway spec_3070 -- --test-threads=1`
- `cargo test -p tau-gateway spec_3068 -- --test-threads=1`
- `cargo test -p tau-gateway spec_3064 -- --test-threads=1`
- `cargo test -p tau-gateway spec_3060 -- --test-threads=1`
- `cargo test -p tau-gateway spec_2921 -- --test-threads=1`
- `cargo test -p tau-gateway spec_2917 -- --test-threads=1`
- `cargo test -p tau-gateway spec_2913 -- --test-threads=1`
- `cargo test -p tau-gateway spec_2909 -- --test-threads=1`
- `cargo test -p tau-gateway spec_2905 -- --test-threads=1`
- `cargo fmt --check`
- `cargo clippy -p tau-dashboard-ui -p tau-gateway -- -D warnings`
