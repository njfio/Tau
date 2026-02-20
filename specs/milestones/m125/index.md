# M125 - Spacebot G10 Discord Thread + Typing Adapter Closure

## Context
`tasks/spacebot-comparison.md` previously left the G10 implementation row unchecked. Tau already supports Discord send/receive, file delivery, reactions, and progressive edits, but it lacked explicit thread creation dispatch and concrete provider typing indicator emission.

## Linked Work
- Epic: #2765
- Story: #2764
- Task: #2766
- Source parity checklist: `tasks/spacebot-comparison.md` (G10)

## Scope
- Add Discord thread creation dispatch in outbound adapter + runtime command integration.
- Add Discord provider typing indicator dispatch in runtime delivery lifecycle.
- Preserve existing non-Discord behavior and long-message fallback semantics.
- Add conformance/regression coverage and checklist evidence.

## Exit Criteria
- `/tau thread <name>` can dispatch Discord thread creation from inbound message context.
- Discord provider mode emits typing indicators through provider API during lifecycle emission.
- Existing Telegram/WhatsApp and Discord long-message fallback behavior remains stable.
- Scoped fmt/clippy/tests and localhost live validation are green.
