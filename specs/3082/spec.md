# Spec: Issue #3082 - ops memory-graph edge style relation contracts

Status: Reviewed

## Problem Statement
`/ops/memory-graph` currently exposes deterministic node/edge contracts, node-size,
and node-color markers, but lacks deterministic edge-style markers by relation type.
PRD checklist contract `2090` remains unverified.

## Scope
In scope:
- Add deterministic edge-style marker attributes for memory graph edges.
- Map relation types to stable style token and dasharray values.
- Validate edge-style rendering via UI and gateway conformance tests.

Out of scope:
- Node interactivity (click/hover/zoom/pan/filter behavior).
- New dependencies.

## Acceptance Criteria
### AC-1 `/ops/memory-graph` preserves deterministic graph contract surface
Given `/ops/memory-graph` renders,
when graph edges are empty,
then deterministic graph panel/list markers remain stable.

### AC-2 Edge style markers reflect relation type
Given memory graph edges with different relation types,
when `/ops/memory-graph` is rendered,
then each edge exposes deterministic style token and dasharray markers mapped from relation type.

### AC-3 Non-memory-graph routes preserve hidden graph contracts
Given any non-memory-graph route renders,
when shell HTML is produced,
then graph panel markers remain present and hidden.

### AC-4 Existing memory graph and memory explorer contracts remain green
Given existing memory graph/explorer specs,
when edge-style contracts are added,
then selected conformance/regression suites remain green.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
|---|---|---|---|---|---|
| C-01 | AC-1 | Functional | `/ops/memory-graph` with no edge rows | render route | graph markers remain deterministic with zero-count defaults |
| C-02 | AC-2 | Integration | edge rows contain multiple relation types | render route | each edge row includes deterministic `data-edge-style-token` and `data-edge-stroke-dasharray` |
| C-03 | AC-3 | Regression | route is not `/ops/memory-graph` | render route | graph panel markers remain present and hidden |
| C-04 | AC-4 | Regression | existing memory specs | rerun selected suites | prior contracts remain green |

## Success Metrics / Signals
- `cargo test -p tau-dashboard-ui spec_3082 -- --test-threads=1`
- `cargo test -p tau-gateway spec_3082 -- --test-threads=1`
- `cargo test -p tau-dashboard-ui spec_3078 -- --test-threads=1`
- `cargo test -p tau-dashboard-ui spec_3070 -- --test-threads=1`
- `cargo test -p tau-dashboard-ui spec_3068 -- --test-threads=1`
- `cargo test -p tau-dashboard-ui spec_3064 -- --test-threads=1`
- `cargo test -p tau-dashboard-ui spec_3060 -- --test-threads=1`
- `cargo test -p tau-dashboard-ui spec_2921 -- --test-threads=1`
- `cargo test -p tau-dashboard-ui spec_2917 -- --test-threads=1`
- `cargo test -p tau-dashboard-ui spec_2913 -- --test-threads=1`
- `cargo test -p tau-dashboard-ui spec_2909 -- --test-threads=1`
- `cargo test -p tau-dashboard-ui spec_2905 -- --test-threads=1`
- `cargo test -p tau-gateway spec_3078 -- --test-threads=1`
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
