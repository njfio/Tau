# Plan: Issue #2766 - Discord thread creation command and provider typing dispatch (G10)

## Approach
1. Add RED tests in outbound/runtime for thread creation, command integration, and typing indicator provider dispatch.
2. Implement new outbound dispatcher methods for `deliver_thread` and `deliver_typing_indicator` (Discord support, fail-closed semantics).
3. Extend runtime command model/parser to support `/tau thread <name>` and dispatch path.
4. Wire typing lifecycle path to provider typing dispatch when enabled.
5. Run scoped gates + live validation; update checklist evidence.

## Affected Modules
- `crates/tau-multi-channel/src/multi_channel_outbound.rs`
- `crates/tau-multi-channel/src/multi_channel_runtime.rs`
- `crates/tau-multi-channel/src/multi_channel_runtime/tests.rs`
- `tasks/spacebot-comparison.md`

## Risks / Mitigations
- Risk: command parser regressions for existing `/tau` subcommands.
  - Mitigation: extend parser tests and run existing command regression suite.
- Risk: typing dispatch failures could break normal response delivery.
  - Mitigation: on typing dispatch failure, log structured failure and continue response delivery.
- Risk: Discord thread endpoint payload/path mismatch.
  - Mitigation: integration tests with exact mock endpoint/body assertions.

## Interfaces / Contracts
- Runtime command additions:
  - `/tau thread <name>`
- Outbound dispatcher additions:
  - thread delivery API
  - typing indicator delivery API
- No CLI flags or wire-format changes.

## ADR
- Not required: no dependency/protocol/architecture change.
