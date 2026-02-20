# Plan: Issue #2893 - sessions list metadata contracts

## Approach
1. Add RED UI and gateway tests asserting deterministic metadata markers on `/ops/sessions` rows.
2. Extend sessions row snapshot contracts in `tau-dashboard-ui` to carry row metadata fields.
3. Extend gateway sessions discovery to load each session store and derive metadata (entry count, usage total tokens, validation status, updated timestamp).
4. Render metadata markers on sessions list rows while preserving existing route/selection contracts.
5. Run required regression + verification gates (fmt/clippy/scoped tests/mutation/live validation).

## Affected Modules
- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-gateway/src/gateway_openresponses/ops_dashboard_shell.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`

## Risks and Mitigations
- Risk: sessions metadata derivation adds filesystem/store read overhead.
  - Mitigation: derive only lightweight per-session summary fields and degrade gracefully on read errors.
- Risk: existing selector/list tests become brittle due new attributes.
  - Mitigation: keep prior marker contracts unchanged; add additive assertions only.
- Risk: oversized-file guardrail in gateway router file.
  - Mitigation: keep changes scoped to `ops_dashboard_shell.rs` and tests.

## Interface / Contract Notes
- No new external API endpoints.
- Existing sessions list row contracts remain additive-only.
- Metadata contract fields are rendered as deterministic HTML `data-*` attributes.
