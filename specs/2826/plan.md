# Plan: Issue #2826 - Command-center control confirmation SSR markers

## Approach
1. Add failing UI and gateway conformance tests for control action confirmation marker contracts.
2. Extend command-center control button rendering in `tau-dashboard-ui` with deterministic confirmation payload attributes.
3. Verify gateway `/ops` integration output includes new confirmation markers.
4. Run phase-1A..1K regressions and validation gates.

## Affected Modules
- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`

## Risks & Mitigations
- Risk: marker additions change existing control button output and break prior assertions.
  - Mitigation: append additive attributes while preserving existing ids and action markers.
- Risk: copy-paste confirmation payload drift between actions.
  - Mitigation: define deterministic per-action strings and assert exact marker contracts in tests.

## Interfaces / Contracts
Per control action button add deterministic marker attributes:
- `data-confirm-required`
- `data-confirm-title`
- `data-confirm-body`
- `data-confirm-verb`

Existing contracts preserved:
- `id="tau-ops-control-action-{pause|resume|refresh}"`
- `data-action-enabled`
- `data-action`

## ADR
No ADR required: no dependency/protocol/architecture boundary change.
