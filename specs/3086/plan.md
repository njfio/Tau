# Plan: Issue #3086 - ops memory-graph node detail panel contracts

## Approach
1. Add RED UI tests for graph node selection/detail-href and graph detail panel markers.
2. Add RED gateway integration tests for graph-route selected-node detail marker behavior.
3. Implement deterministic node selection + detail-href + graph detail panel rendering contracts.
4. Run regression suites for memory graph/explorer contracts and verification gates.

## Affected Modules
- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-dashboard-ui/src/tests.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`

## Risks and Mitigations
- Risk: detail contracts may drift between memory and memory-graph route semantics.
  - Mitigation: derive graph detail markers directly from existing selected-memory snapshot fields.
- Risk: new node attributes could regress existing size/color/style contracts.
  - Mitigation: rerun `spec_3082`, `spec_3078`, `spec_3070`, and baseline memory suites.

## Interface / Contract Notes
- Extend memory graph node rows with deterministic selection/detail markers.
- Add a dedicated graph-route detail panel marker surface:
  - `#tau-ops-memory-graph-detail-panel`
- No external API additions; contracts are SSR marker-level only.
- P1 process rule: spec marked Reviewed; human review requested in PR.
