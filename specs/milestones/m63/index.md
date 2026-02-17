# M63 - Spacebot G12 Skip Tool

Milestone objective: deliver a deterministic skip-response path for multi-channel `/tau`
commands so operators (and later agent tool wiring) can intentionally suppress outbound
delivery for one event while preserving auditable command metadata.

## Scope
- Parse and route `/tau skip [reason]` as a first-class command.
- Suppress outbound transport delivery for explicit skip requests.
- Persist auditable outbound log metadata for skipped events.
- Conformance tests for parser, runtime behavior, and regression safety.

## Out of Scope
- Full LLM-invoked `skip` agent tool orchestration across all runtimes.
- Platform-specific reaction/file tooling from other Spacebot parity items.
- Changes to provider integrations, auth flows, or routing policy models.

## Exit Criteria
- Issue `#2383` AC/C-case mapping implemented.
- Mapped tests pass in `tau-multi-channel`.
- Parent hierarchy (`#2381` → `#2384`) is closed with status labels updated.
