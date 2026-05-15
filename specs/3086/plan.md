# Plan: Issue #3086 - ops memory-graph node detail panel contracts

## Approach
1. Add RED UI tests for graph node selection/detail-href, graph detail panel markers, selected proof, and connected-relation row markers.
2. Add RED gateway integration tests for graph-route selected-node detail marker behavior, including relation proof.
3. Add gateway regression coverage for selected ops-harness lineage nodes that are not persisted memory records.
4. Implement deterministic node selection + detail-href + graph/detail-panel rendering contracts for persisted and lineage nodes.
5. Run regression suites for memory graph/explorer contracts and verification gates.

## Affected Modules
- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-dashboard-ui/src/tests.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`

## Risks and Mitigations
- Risk: detail contracts may drift between memory and memory-graph route semantics.
  - Mitigation: derive graph detail markers directly from existing selected-memory snapshot fields.
- Risk: new node attributes could regress existing size/color/style contracts.
  - Mitigation: rerun `spec_3082`, `spec_3078`, `spec_3070`, and baseline memory suites.
- Risk: ops-harness overlay nodes are synthetic and do not have embedding vectors.
  - Mitigation: mark them with deterministic `ops-harness-lineage` embedding-source/reason-code fields and expose only label/category/outgoing relations.

## Interface / Contract Notes
- Extend memory graph node rows with deterministic selection/detail markers.
- Add a dedicated graph-route detail panel marker surface:
  - `#tau-ops-memory-graph-detail-panel`
- Graph-route detail panels expose `#tau-ops-memory-graph-detail-proof`, `#tau-ops-memory-graph-detail-relation-scope`, and `#tau-ops-memory-graph-detail-relations` so selected id/type and stored-vs-graph relation counts are visible without leaving the graph page.
- If selected ID is not in `tau-memory`, fall back to the ops-harness lineage overlay before clearing the selection.
- No external API additions; contracts are SSR marker-level only.
- P1 process rule: spec marked Reviewed; human review requested in PR.
