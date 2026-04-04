# Plan: Issue #3690 - Add placeholder guardrails to `tau-tui` command palette scaffolds

Status: Implemented

## Approach
Intercept command palette submission after scaffold resolution but before command
execution. If the resolved command still contains placeholder markers such as
`<mission-id>`, block the execution path, keep the operator in command mode, and
emit a clear guidance message describing which placeholders remain unresolved.

## Affected Modules
- `crates/tau-tui/src/interactive/app_commands.rs`
  - detect unresolved placeholders during palette submission
  - emit operator guidance instead of executing unresolved scaffolds
- `crates/tau-tui/src/interactive/app_gateway_tests.rs`
  - RED/GREEN tests for blocked scaffold execution, guidance, and real-value
    execution

## Contracts
- Commands containing unresolved `<...>` placeholders do not execute
- Guidance names the unresolved placeholder tokens
- Commands with real values continue through normal execution

## Risks
- Placeholder detection should not catch legitimate literal angle-bracket input
  outside the scaffold use case
- Blocked submissions should not unexpectedly close the palette or lose input
- The guidance message should be clear without becoming verbose

## Verification Strategy
- Add failing tests first for unresolved scaffold blocking and guidance
- Re-run `interactive::app_gateway_tests`
- Run `rustfmt --check`
- Build `tau-tui`
