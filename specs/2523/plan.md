# Plan #2523

## Approach
1. Add `SendFileTool` to `tau-tools` and register in built-ins.
2. Add send-file directive extraction + suppression in `tau-agent-core` for successful tool results.
3. Extend `tau-coding-agent` event payload logic to include send-file metadata and reason code while suppressing reply text.
4. Add conformance/regression tests and run scoped mutation + live validation.

## Affected Modules
- `crates/tau-tools/src/tools.rs`
- `crates/tau-tools/src/tools/registry_core.rs`
- `crates/tau-tools/src/tools/tests.rs`
- `crates/tau-agent-core/src/lib.rs`
- `crates/tau-agent-core/src/tests/config_and_direct_message.rs`
- `crates/tau-coding-agent/src/events.rs`

## Risks / Mitigations
- Risk: false-positive suppression for malformed payloads.
  - Mitigation: strict parser checks + regression tests for malformed/error payloads.
- Risk: regression in existing send-file command path.
  - Mitigation: keep C-06 functional test in verification matrix.

## Interface Contracts
- Tool name: `send_file`
- Arguments: `file_path` (required), `message` (optional)
- Tool success payload: `send_file_response`, `file_path`, optional `message`, `reason_code`, `suppress_response`.
