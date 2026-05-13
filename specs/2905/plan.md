# Plan: Issue #2905 - ops memory search relevant result contracts

## Approach
1. Add RED UI tests for memory search panel/form markers, query preservation, and empty-state/result counters.
2. Add RED gateway integration tests that seed persisted memory entries and assert relevant `/ops/memory` result rows.
3. Implement minimal memory snapshot plumbing in ops shell and memory panel rendering contracts in `tau-dashboard-ui`.
4. Expose graph node/edge availability in the Memory Scope summary and empty-state row so empty search rows are not confused with an empty memory graph.
5. Render a bounded graph-backed node preview on zero-result pages so operators can inspect memory summaries and node metadata before jumping to Memory Graph.
6. Run regression + verify gates (fmt/clippy/spec slices/mutation/live validation).

## Affected Modules
- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-gateway/src/gateway_openresponses/ops_shell_controls.rs`
- `crates/tau-gateway/src/gateway_openresponses/ops_dashboard_shell.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`

## Risks and Mitigations
- Risk: adding new snapshot fields can break many existing shell tests.
  - Mitigation: add defaults and additive rendering markers only; preserve existing marker IDs.
- Risk: memory search relevance can vary by backend.
  - Mitigation: seed deterministic entries and assert target entry IDs/summary fragments.
- Risk: route-level regressions in chat/session panels.
  - Mitigation: rerun established chat/session/detail regression slices.

## Interface / Contract Notes
- No new HTTP endpoints.
- No protocol/schema changes.
- Additive SSR marker contracts on existing `/ops/memory` route.
- Memory Scope and the empty-state row include additive `data-graph-node-count`, `data-graph-edge-count`, and `data-graph-state` markers plus visible/copy evidence that graph-backed memory is still available.
- Zero-result Memory Explorer pages render a bounded graph-backed preview with memory summaries, node ids, types, importance, and graph detail links.
