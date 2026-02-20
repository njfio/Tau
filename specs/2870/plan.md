# Plan: Issue #2870 - chat markdown and code-block contracts

## Approach
1. Add additive chat transcript SSR markers in `tau-dashboard-ui` for assistant markdown rows and fenced code blocks.
2. Add failing UI and gateway tests for `/ops`, `/ops/chat`, and `/ops/sessions` route contracts.
3. Re-run chat regression suites and contract verification gates.

## Affected Modules
- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`

## Risks and Mitigations
- Risk: changing row rendering could break existing selectors.
  - Mitigation: keep existing row IDs/attributes and add nested additive marker elements only.
- Risk: deterministic attribute payload could drift due whitespace handling.
  - Mitigation: normalize extracted code block text before setting marker attributes and lock with tests.

## Interface / Contract Notes
- SSR markup contract only.
- No protocol/API/schema changes.
- No new dependencies.
