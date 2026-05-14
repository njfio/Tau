# Plan: Issue #2905 - ops memory search relevant result contracts

## Approach
1. Add RED UI tests for memory search panel/form markers, query preservation, and empty-state/result counters.
2. Add RED gateway integration tests that seed persisted memory entries and assert relevant `/ops/memory` result rows.
3. Implement minimal memory snapshot plumbing in ops shell and memory panel rendering contracts in `tau-dashboard-ui`.
4. Expose graph node/edge availability in the Memory Scope summary and empty-state row so empty search rows are not confused with an empty memory graph.
5. Render a bounded graph-backed node preview on zero-result pages so operators can inspect memory summaries plus node/relation metadata, distinguish a not-run search from a no-match search, see the search-result count and graph-preview availability as separate scope metrics, follow sample relations into Memory Graph, return to the originating preview row context, see the returned row marked as selected, recover when the returned memory is outside the bounded preview, distinguish missing return ids from filtered-out return ids, and see an explicit Memory Graph not-found detail state when a recovery `detail_memory_id` cannot be resolved.
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
- Zero-result Memory Explorer pages render a bounded graph-backed preview with memory summaries, node ids, types, importance, connected relation counts, sample relation links, and graph detail links.
- Zero-result Memory Explorer pages expose `data-search-state` and copy that distinguishes no active query from an executed query with no matching search rows.
- The Memory Scope summary labels the result count as search results and exposes additive graph-preview count/limit markers so graph-backed entries do not contradict a zero search-result count.
- Memory Graph detail links back to Memory Explorer carry additive `preview_memory_id` and preview-row anchor contracts when the selected node is present in the bounded preview.
- Memory Explorer consumes additive `preview_memory_id` query state and marks the matching bounded preview row with selected DOM markers and visible returned-from-graph copy.
- If `preview_memory_id` is outside the bounded preview, Memory Explorer renders additive out-of-preview markers, a visible explanation, and a Memory Graph recovery link for the selected memory.
- If `preview_memory_id` is filtered out of the current graph scope, Memory Explorer renders filtered-out recovery markers and an unfiltered Memory Graph link.
- If `preview_memory_id` is missing even from the unfiltered graph scope, Memory Explorer renders not-found recovery markers and copy instead of suggesting another unfiltered attempt.
- If Memory Graph receives a `detail_memory_id` that cannot be found in memory store or harness lineage, it keeps the selected detail hidden while exposing additive `data-detail-requested`, `data-detail-state="not-found"`, and `data-requested-memory-id` markers plus visible not-found copy.
