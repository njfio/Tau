# Plan: Issue #2393 - Implement /tau send-file dispatch and audit logging

## Approach
1. Extend `MultiChannelTauCommand` with `SendFile` and parser/help/renderer wiring.
2. Extend `MultiChannelCommandExecution` with send-file metadata (URL + optional caption).
3. Add outbound dispatcher API `deliver_file(...)` with transport-specific request shaping.
4. In suppressed command path, call `deliver_file(...)`, map errors to stable command reason codes,
   and persist deterministic outbound payloads.
5. Add conformance tests first, then implement minimal production changes.

## Affected Modules
- `crates/tau-multi-channel/src/multi_channel_runtime.rs`
- `crates/tau-multi-channel/src/multi_channel_runtime/tests.rs`
- `crates/tau-multi-channel/src/multi_channel_outbound.rs`

## Risks and Mitigations
- Risk: file delivery APIs vary significantly by transport.
  - Mitigation: implement initial deterministic slice (Telegram support) and fail-closed on unsupported transports with explicit reason codes.
- Risk: command suppression branch regresses skip/react behavior.
  - Mitigation: keep suppression framework shared and add regression tests.
- Risk: mutation escapes in reason-code mapping.
  - Mitigation: assert exact `reason_code` fields in failure tests.

## Interface/Contract Notes
- New command syntax: `/tau send-file <https-url> [caption]`.
- Command payload schema remains `multi_channel_tau_command_v1` with optional fields:
  `send_file_url`, `send_file_caption`.
- Suppressed command logs continue to use outbound `status` + `reason_code` fields.

## ADR
- No dependency/protocol redesign in this slice; ADR not required.
