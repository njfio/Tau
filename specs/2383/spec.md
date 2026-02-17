# Spec: Issue #2383 - Implement SkipTool and suppress outbound delivery

Status: Accepted

## Problem Statement
The multi-channel runtime always produces an outbound user-visible response for `/tau`
commands. We need an explicit skip-response command path so an operator can intentionally
end a turn without sending outbound text, while still recording why suppression occurred.

## Acceptance Criteria

### AC-1 Parse `/tau skip [reason]` as a first-class command
Given inbound text beginning with `/tau skip`,
When command parsing executes,
Then it returns a typed skip command with optional reason text.

### AC-2 Skip command suppresses outbound delivery
Given a valid skip command event,
When `run_once_events` processes the event,
Then outbound dispatcher delivery is not invoked and no assistant response text is appended.

### AC-3 Skip command is auditable
Given a valid skip command event,
When processing completes,
Then channel logs include a deterministic outbound status entry documenting skip reason/code
and command payload metadata.

### AC-4 Existing command behavior is preserved
Given existing `/tau` commands (for example `/tau status`),
When processed after this change,
Then command response and outbound delivery behavior remains unchanged.

## Scope

### In Scope
- Multi-channel command parser support for `skip`.
- Runtime command execution + outbound suppression wiring for skip.
- Deterministic outbound status logging for skip path.
- Regression protection for non-skip command flows.

### Out of Scope
- Generic LLM tool invocation integration for skip across agent-core.
- New transport adapters or message formatting redesign.

## Conformance Cases

| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit | `parse_multi_channel_tau_command(\"/tau skip too noisy\")` | Returns `Some(Skip { reason: \"too noisy\" })` |
| C-02 | AC-2/AC-3 | Functional | `run_once_events` with one `/tau skip maintenance` event | Outbound skip status log exists; no response delivery payload/context assistant text |
| C-03 | AC-4 | Regression | `run_once_events` with `/tau status` | Existing command response payload and delivery metadata still present |
| C-04 | AC-1 | Unit | `render_multi_channel_tau_command_help()` | Help text lists `/tau skip [reason]` |

## Success Metrics / Observable Signals
- C-01..C-04 tests pass in `tau-multi-channel`.
- Skip events produce auditable log records with stable reason codes.
- Non-skip `/tau` command tests remain green.
