# M124 - Spacebot G10 Discord Placeholder Streaming Edits

## Context
`tasks/spacebot-comparison.md` still leaves G10 `Message streaming via placeholder message + progressive edits` unchecked. Tau outbound Discord provider delivery currently posts direct final chunks without placeholder/edit progression.

## Linked Work
- Epic: #2760
- Story: #2761
- Task: #2762
- Source parity checklist: `tasks/spacebot-comparison.md` (G10)

## Scope
- Add Discord provider-mode outbound placeholder send + progressive edit behavior.
- Preserve existing non-Discord and over-2000-char fallback behavior.
- Add conformance/regression coverage and update parity evidence.

## Exit Criteria
- Discord provider mode uses placeholder create + progressive PATCH edits for stream-compatible responses.
- Final edited Discord content equals full response text.
- Messages exceeding Discord max content continue safe chunked behavior.
- Scoped fmt/clippy/tests and localhost validation are green.
