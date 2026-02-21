# Plan: Issue #2988 - channel lifecycle and telemetry runtime extraction

## Approach
1. Record RED baseline size and scoped tests.
2. Create `channel_telemetry_runtime.rs` and move handlers + dedicated helper plumbing.
3. Wire imports and keep existing route registration unchanged.
4. Run scoped regression, quality gates, and sanitized live validation.

## Affected Modules
- `crates/tau-gateway/src/gateway_openresponses.rs`
- `crates/tau-gateway/src/gateway_openresponses/channel_telemetry_runtime.rs`

## Risks and Mitigations
- Risk: telemetry persistence path behavior drift.
  - Mitigation: pure move and rerun telemetry endpoint tests.
- Risk: lifecycle command config drift.
  - Mitigation: move helper unchanged and validate with lifecycle tests.

## Interfaces / Contracts
- No path changes.
- No payload contract changes.
