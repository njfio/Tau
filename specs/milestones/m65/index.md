# M65 - Spacebot G14 Send File Tool

Milestone objective: deliver a deterministic file-delivery command path for multi-channel
`/tau` commands so operators can send file artifacts without emitting a normal text reply,
while preserving auditable metadata and stable failure reason codes.

## Scope
- Parse and route `/tau send-file <https-url> [caption]` as a first-class command.
- Dispatch outbound file delivery through multi-channel transport adapters.
- Suppress normal outbound response delivery for send-file commands.
- Persist auditable command and delivery metadata for success/failure.
- Conformance tests for parser/help/runtime behavior and regression safety.

## Out of Scope
- Local filesystem upload from channel users.
- Slack-specific runtime wiring for file upload flows.
- Dashboard/UI file browser features.

## Exit Criteria
- Issue `#2393` AC/C-case mapping implemented.
- Tests pass for `tau-multi-channel` send-file command and outbound delivery paths.
- Parent hierarchy (`#2391` -> `#2394`) is closed with status labels updated.
