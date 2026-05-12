# Plan: Issue #2870 - chat markdown and code-block contracts

## Approach
1. Add additive chat transcript SSR markers in `tau-dashboard-ui` for assistant markdown rows and fenced code blocks.
2. Add failing UI and gateway tests for `/ops/chat` visible rendering and `/ops` plus `/ops/sessions` hidden-panel omission contracts.
3. Add a lightweight dependency-free SSR formatter for common assistant markdown and Tau operator-response sections.
4. Re-run chat regression suites and contract verification gates.

## Affected Modules
- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`

## Risks and Mitigations
- Risk: changing row rendering could break existing selectors.
  - Mitigation: keep existing row IDs/attributes and add nested additive marker elements only.
- Risk: deterministic attribute payload could drift due whitespace handling.
  - Mitigation: normalize extracted code block text before setting marker attributes and lock with tests.
- Risk: markdown/code transcript payload leaks through hidden off-route chat panels.
  - Mitigation: explicit `/ops` and `/ops/sessions` assertions preserve the hidden panel shell but omit rendered transcript rows and payload markers.
- Risk: a full Markdown implementation would expand dependencies and parsing surface.
  - Mitigation: keep the formatter scoped to headings, links, bold, inline code, ordered/unordered lists, tables, fenced code, and Tau's current operator-response sections.

## Interface / Contract Notes
- SSR markup contract only; no client-side renderer.
- No protocol/API/schema changes.
- No new dependencies.
