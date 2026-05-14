# Spec: Issue #2905 - ops memory search relevant result contracts

Status: Implemented

## Problem Statement
The PRD requires Memory Explorer search to return relevant results, but the Tau Ops shell currently lacks explicit memory search panel/result contracts and corresponding route-level conformance coverage.

## Scope
In scope:
- Add deterministic `/ops/memory` search panel contracts (form/query/result markers).
- Add gateway integration tests proving persisted memory entries surface as relevant search rows.
- Add empty-state contracts for no-match searches.
- Add graph availability counts to the Memory Scope summary so zero search results do not imply the graph store is empty.
- Add a compact graph-backed preview on zero-result pages so operators can inspect available memory nodes without leaving Memory Explorer.
- Distinguish a not-run search state from a no-match search state when `/ops/memory` has no active query.
- Preserve the graph-backed preview return context when operators follow a preview node or relation into Memory Graph detail and then return to Memory Explorer.
- Mark the returned graph-backed preview row as selected when Memory Explorer receives a `preview_memory_id`.
- Explain and recover from out-of-preview return state when `preview_memory_id` exists in the graph scope but is not present in the bounded preview.
- Distinguish missing or filtered-out `preview_memory_id` values from real out-of-preview nodes.
- When a preview recovery link opens Memory Graph with a missing `detail_memory_id`, render an explicit not-found detail state that preserves the requested memory id.

Out of scope:
- Memory graph route behavior beyond displaying its counts in the Memory Scope summary and the bounded preview recovery detail state.
- Memory entry create/edit/delete UI workflows.
- New dependencies.

## Acceptance Criteria
### AC-1 Memory route exposes deterministic search form contracts
Given `/ops/memory` is rendered,
when an operator loads the route with a search query,
then the memory panel exposes deterministic form/query markers preserving the requested query.

### AC-2 Relevant persisted memory matches render as deterministic result rows
Given persisted memory entries and a matching query,
when `/ops/memory` renders,
then relevant matches appear as deterministic result rows with stable metadata contracts.

### AC-3 No-match searches expose empty-state contracts
Given a search query with no matches,
when `/ops/memory` renders,
then a deterministic empty-state marker is shown and result row count is zero.

### AC-3a Memory Scope distinguishes search rows from graph availability
Given `/ops/memory` has zero search result rows but graph rows are available,
when `/ops/memory` renders,
then the Memory Scope summary and empty-state row expose deterministic search-result, graph-preview, graph node/edge counts and graph state markers, the empty-state copy states whether search is not active or no matching search rows were found, a bounded graph-backed node preview renders memory summaries with node metadata plus navigable relation samples, Memory Graph detail return links preserve the originating preview memory and row anchor when available, the returned preview row is visibly and deterministically marked as selected, a real out-of-preview `preview_memory_id` renders a visible explanation plus recovery link, a missing or filtered-out `preview_memory_id` renders a distinct not-in-scope explanation without implying the memory exists, and the Memory Graph recovery route renders an explicit not-found detail state when the requested `detail_memory_id` cannot be resolved.

### AC-4 Existing route contracts remain intact
Given existing ops/chat/sessions/detail contracts,
when memory search contracts are added,
then existing suites remain green.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
|---|---|---|---|---|---|
| C-01 | AC-1 | Functional | `/ops/memory?query=<q>` | render memory route | form/action/query markers are present and query preserved |
| C-02 | AC-2 | Integration | persisted memory entries containing query terms | render `/ops/memory?query=<q>` | deterministic result rows include relevant entries |
| C-03 | AC-3 | Functional | query with no matches | render memory route | empty-state marker present and result count zero |
| C-03a | AC-3a | Functional/Integration | zero search rows with graph rows available | render memory route without a query, render a no-match query, follow a preview node/relation into graph detail, then return with `preview_memory_id` or recover through Memory Graph with a missing `detail_memory_id` | scope summary labels the count as search results, shows graph-preview count/limit plus graph node/edge counts, graph state, `not-run` versus no-match search state, bounded graph-backed node preview with summaries plus relation sample links, graph detail return links include `preview_memory_id` plus the preview row anchor, the returned preview row carries selected markers plus visible selected copy, real out-of-preview return state includes an explanation plus Memory Graph recovery link, missing or filtered-out return state uses distinct not-in-scope markers/copy, and the Memory Graph recovery route preserves the requested id in a not-found detail state |
| C-04 | AC-4 | Regression | existing contracts | rerun selected suites | chat/session/dashboard contracts remain green |

## Success Metrics / Signals
- `cargo test -p tau-dashboard-ui spec_2905 -- --test-threads=1` passes.
- `cargo test -p tau-gateway spec_2905 -- --test-threads=1` passes.
- Required regression slices remain green.
