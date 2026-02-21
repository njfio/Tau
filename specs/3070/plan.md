# Plan: Issue #3070 - ops memory-graph node size importance contracts

## Approach
1. Add RED UI tests for node-size marker contracts on memory graph rows.
2. Add RED gateway integration tests for low/high importance node-size derivation.
3. Implement importance normalization and deterministic size marker rendering.
4. Run regression suites for memory graph/explorer contracts and verification gates.

## Affected Modules
- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-dashboard-ui/src/tests.rs`
- `crates/tau-gateway/src/gateway_openresponses/ops_dashboard_shell.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`

## Risks and Mitigations
- Risk: size derivation may be unstable around boundaries.
  - Mitigation: deterministic bucket thresholds and explicit clamping.
- Risk: graph marker changes may regress prior node/edge contracts.
  - Mitigation: rerun `spec_3068` and memory explorer regression suites.

## Interface / Contract Notes
- Extend memory graph node row markers with `data-node-size-bucket` and
  `data-node-size-px`.
- No external API additions; contracts are SSR marker-level only.
- P1 process rule: spec marked Reviewed; human review requested in PR.
