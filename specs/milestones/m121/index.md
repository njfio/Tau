# M121 - Spacebot G10 Discord Permission Filtering

## Context
`tasks/spacebot-comparison.md` still leaves G10 `Guild/channel filtering for permissions` unchecked. Tau already supports Discord polling and channel-id selection, but it lacks explicit guild allowlist controls in live connector ingress.

## Linked Work
- Epic: #2748
- Story: #2749
- Task: #2750
- Source parity checklist: `tasks/spacebot-comparison.md` (G10)

## Scope
- Add Discord guild allowlist CLI/config support for live connector polling mode.
- Enforce guild filtering before Discord message ingestion.
- Preserve existing channel allowlist behavior and default compatibility when no guild allowlist is configured.
- Add conformance/regression coverage and update roadmap evidence.

## Exit Criteria
- Operators can configure Discord guild allowlists for live ingress polling.
- Discord messages outside allowlisted guilds are ignored before ingestion.
- Existing Discord polling behavior remains unchanged when guild allowlist is unset.
- Scoped fmt/clippy/tests and localhost live validation are green.
