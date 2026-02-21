# Spec: Issue #3086 - ops memory-graph node detail panel contracts

Status: Reviewed

## Problem Statement
`/ops/memory-graph` exposes deterministic node/edge contracts (including size/color/style),
but lacks deterministic contracts that represent node selection and graph-route detail panel
visibility. PRD checklist contract `2091` remains unverified.

## Scope
In scope:
- Add deterministic node-selection and detail-href markers for memory graph nodes.
- Add deterministic graph-route detail panel markers bound to selected memory entry.
- Validate node selection/detail contracts via UI and gateway conformance tests.

Out of scope:
- Hover, zoom, pan, and filter interactivity contracts.
- New dependencies.

## Acceptance Criteria
### AC-1 `/ops/memory-graph` preserves deterministic graph contract surface
Given `/ops/memory-graph` renders,
when no node is selected,
then graph panel contracts remain deterministic and detail panel markers are hidden/default.

### AC-2 Selected node contracts expose detail-panel behavior
Given a selected memory entry ID is provided on `/ops/memory-graph`,
when shell HTML is rendered,
then node rows expose deterministic selection/detail-href markers and graph detail panel markers reflect the selected entry.

### AC-3 Non-memory-graph routes preserve hidden graph contracts
Given any non-memory-graph route renders,
when shell HTML is produced,
then graph panel markers remain present and hidden.

### AC-4 Existing memory graph and memory explorer contracts remain green
Given existing memory graph/explorer specs,
when node-detail contracts are added,
then selected conformance/regression suites remain green.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
|---|---|---|---|---|---|
| C-01 | AC-1 | Functional | `/ops/memory-graph` without selected node | render route | detail panel markers remain present with hidden/default values |
| C-02 | AC-2 | Integration | selected detail memory ID on graph route | render route | selected node row exposes detail markers and graph detail panel mirrors selected entry |
| C-03 | AC-3 | Regression | route is not `/ops/memory-graph` | render route | graph panel markers remain present and hidden |
| C-04 | AC-4 | Regression | existing memory specs | rerun selected suites | prior contracts remain green |

## Success Metrics / Signals
- `cargo test -p tau-dashboard-ui spec_3086 -- --test-threads=1`
- `cargo test -p tau-gateway spec_3086 -- --test-threads=1`
- `cargo test -p tau-dashboard-ui spec_3082 -- --test-threads=1`
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
- `cargo test -p tau-gateway spec_3082 -- --test-threads=1`
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
