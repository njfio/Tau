# Spec: Issue #2393 - Implement /tau send-file dispatch and audit logging

Status: Accepted

## Problem Statement
Tau supports `/tau` command operations for status/skip/react but cannot send file artifacts as a
first-class command. Users are forced to post links manually or send text-only responses, which
misses the G14 parity target and prevents lightweight file delivery workflows.

## Acceptance Criteria

### AC-1 Parse `/tau send-file <https-url> [caption]` as a first-class command
Given inbound text beginning with `/tau send-file`,
When command parsing executes,
Then it returns a typed send-file command with validated HTTPS URL and optional caption.

### AC-2 Send-file command dispatches file delivery and suppresses text response
Given a valid send-file command event on a supported transport,
When `run_once_events` processes the event,
Then file dispatch is attempted and no outbound assistant text response is delivered.

### AC-3 Send-file command outcomes are auditable
Given a processed send-file command event,
When runtime persistence completes,
Then outbound logs include deterministic command metadata (url, caption, status/reason)
and delivery outcome payload.

### AC-4 Unsupported/invalid send-file targets fail with stable reason codes
Given a send-file command with unsupported transport or invalid URL,
When runtime processes the event,
Then command execution records failed status with stable reason codes and suppresses text output.

### AC-5 Existing `/tau` command behavior remains unchanged
Given existing `/tau` commands (`status`, `skip`, `react`),
When processed after this change,
Then current behavior and log contracts remain unchanged.

## Scope

### In Scope
- `MultiChannelTauCommand` parser/renderer/help updates for `send-file`.
- Runtime send-file execution + suppression wiring.
- Outbound dispatcher file delivery API for initial supported transport slice.
- Deterministic success/failure reason codes + command payload fields.

### Out of Scope
- Arbitrary local path uploads from channel messages.
- Multi-provider storage integration.
- New transport adapters beyond currently supported multi-channel transports.

## Conformance Cases

| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit | `parse_multi_channel_tau_command("/tau send-file https://example.com/report.pdf Q1 report")` | Returns `Some(SendFile { url: "https://example.com/report.pdf", caption: Some("Q1 report") })` |
| C-02 | AC-1 | Unit | `render_multi_channel_tau_command_help()` | Help includes `/tau send-file <https-url> [caption]` |
| C-03 | AC-2/AC-3 | Functional | `run_once_events` with Telegram `/tau send-file https://example.com/report.pdf Q1` in dry-run mode | Outbound status `sent_file` + command metadata + delivery payload; no assistant context |
| C-04 | AC-4 | Functional | `run_once_events` with WhatsApp `/tau send-file https://example.com/report.pdf` | Outbound status `failed` with stable reason code and no assistant context |
| C-05 | AC-5 | Regression | Existing `/tau react 👍 42` flow | Existing react suppression behavior remains unchanged |

## Success Metrics / Observable Signals
- C-01..C-05 tests pass in `tau-multi-channel`.
- Send-file command logs include deterministic reason/status and URL metadata.
- Existing command-path tests remain green.
