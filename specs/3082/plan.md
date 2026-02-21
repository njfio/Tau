# Plan: Issue #3082 - ops memory-graph edge style relation contracts

## Approach
1. Add RED UI tests for edge-style marker contracts on memory graph edge rows.
2. Add RED gateway integration tests for relation-type edge-style mappings.
3. Implement deterministic relation-type -> style token/dasharray mapping and marker rendering.
4. Run regression suites for memory graph/explorer contracts and verification gates.

## Affected Modules
- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-dashboard-ui/src/tests.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`

## Risks and Mitigations
- Risk: relation aliases (`supports`, `depends_on`, `blocks`) drift from canonical style mapping.
  - Mitigation: normalize relation values in one UI helper with explicit alias coverage.
- Risk: edge marker additions could regress existing node contracts.
  - Mitigation: rerun `spec_3078`, `spec_3070`, and memory graph/explorer regression suites.

## Interface / Contract Notes
- Extend memory graph edge rows with `data-edge-style-token` and
  `data-edge-stroke-dasharray` markers.
- No external API additions; contracts are SSR marker-level only.
- P1 process rule: spec marked Reviewed; human review requested in PR.
