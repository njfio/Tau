# 3607 Live Mutating Tool Progress

## Objective
Make the interactive TUI show unmistakable live evidence when Tau is making mutating progress during build/create turns, especially for `write` and `edit` tool activity.

## Inputs/Outputs
- Inputs:
  - current agent state from `StatusBar`
  - latest user prompt in the active turn
  - current-turn `ToolEntry` records
- Outputs:
  - live summary lines in the interactive chat/status surfaces that distinguish:
    - no mutating evidence yet
    - read-only progress
    - mutating progress in flight
    - latest successful mutating target path when available

## Boundaries/Non-goals
- No runtime/provider/tool execution changes.
- No new tool payload fields.
- No full TUI layout redesign.
- No webchat changes.

## Failure Modes
- Build/create turns incorrectly show mutating progress for read-only tools.
- Non-build turns show mutating progress lines.
- Prior-turn write/edit success leaks into the next turn.
- Missing tool detail produces malformed or misleading path output.

## Acceptance Criteria
- [ ] Active build/create turn with a running `write` or `edit` tool shows a distinct mutating-progress line.
- [ ] Active build/create turn with a successful `write` or `edit` tool shows the latest mutating target path when detail is present.
- [ ] Active build/create turn that only has read-only tool activity does not show mutating-in-flight wording.
- [ ] Idle turns and non-build turns omit the mutating-progress lines.
- [ ] Integration coverage exercises a render path that transitions from read-only to mutating activity within one build turn.

## Files To Touch
- `crates/tau-tui/src/interactive/ui_chat_tool_lines.rs`
- `crates/tau-tui/src/interactive/tools.rs`
- `crates/tau-tui/src/interactive/ui_tool_visibility_tests.rs`
- `crates/tau-tui/src/interactive/ui_build_status_tests.rs`
- `specs/3607-live-mutating-tool-progress.md`

## Error Semantics
- No silent fallback to mutating wording without matching `write`/`edit` evidence.
- If tool detail is empty, render mutating state without inventing a path.
- Rendering helpers remain pure; no logging or swallowing state mismatches.

## Test Plan
- Add red render tests for:
  - running `write` mutating-progress line
  - successful `write` target path line
  - read-only build turn omits mutating wording
  - new turn reset behavior remains correct
- Run targeted `tau-tui` tests first, then full `cargo test -p tau-tui`.

## Status
- Implemented on branch `3607-live-mutating-tool-progress`.
- No deviations from the original scope.

## Validation Evidence
- `cargo test -p tau-tui 3607 -- --nocapture`
- `cargo test -p tau-tui`
- `cargo clippy -p tau-tui --all-targets --all-features -- -D warnings`
- live smoke: `target/debug/tau-tui interactive --profile ops-interactive`
