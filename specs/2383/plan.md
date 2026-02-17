# Plan: Issue #2383

## Approach
1. Extend `MultiChannelTauCommand` with a `Skip { reason }` variant and parser/help wiring.
2. Add runtime execution handling that marks skip commands as delivery-suppressed.
3. Branch outbound path in event persistence:
   - for skip commands, record outbound skip status + command metadata and return early;
   - for non-skip commands, preserve existing delivery flow unchanged.
4. Add/adjust tests for parser conformance, skip functional behavior, and non-skip regression.

## Affected Modules
- `crates/tau-multi-channel/src/multi_channel_runtime.rs`
- `crates/tau-multi-channel/src/multi_channel_runtime/tests.rs`

## Risks and Mitigations
- Risk: accidental suppression of non-skip commands.
  Mitigation: explicit boolean gating on command execution metadata + regression test for
  `/tau status`.
- Risk: log schema drift for command payload consumers.
  Mitigation: preserve existing command payload keys; add skip-specific fields only.

## Interfaces / Contracts
- `/tau skip [reason]` becomes a supported command in parser/help output.
- Outbound log entry for skip uses status/reason metadata without invoking delivery dispatcher.
- Existing command payload schema remains `multi_channel_tau_command_v1`.

## ADR
No ADR required. This is a local behavior extension in existing command/runtime flow
without dependency or protocol boundary changes.
