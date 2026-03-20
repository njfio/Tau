# 3609 Mutating Transcript Breadcrumbs

## Objective
Make mutating file activity obvious inside the transcript itself and expose a command that copies the latest successful mutating target path from the interactive TUI.

## Inputs/Outputs
- Inputs:
  - transcript tool entries built from `ToolEntry`
  - interactive slash commands from `submit_input` / command palette execution
  - current-turn and latest successful `ToolEntry` state
- Outputs:
  - transcript breadcrumbs for `write` and `edit` with target paths when present
  - `/copy-target` command that copies the latest successful mutating target path
  - visible system error message when no successful mutating target exists

## Boundaries/Non-goals
- No runtime/provider changes.
- No full transcript redesign.
- No exporting full file contents or diffs.
- No webchat changes.

## Failure Modes
- Non-mutating tools render mutating transcript breadcrumbs.
- `/copy-target` silently succeeds when no successful mutating target exists.
- `/copy-target` invents a path when tool detail is empty.
- Prior-turn target state leaks incorrectly after newer successful mutating tool activity.

## Acceptance Criteria
- [ ] Transcript tool entries for `write` and `edit` include a distinct mutating breadcrumb with the target path when detail is present.
- [ ] Transcript tool entries for non-mutating tools do not use the mutating breadcrumb.
- [ ] `/copy-target` copies the latest successful mutating target path and emits a visible success system message.
- [ ] `/copy-target` fails loudly with a visible system message when no successful mutating target exists.
- [ ] Integration coverage exercises the real interactive command path for `/copy-target`.

## Files To Touch
- `crates/tau-tui/src/interactive/app_commands.rs`
- `crates/tau-tui/src/interactive/tools.rs`
- `crates/tau-tui/src/interactive/ui_chat_tool_lines.rs`
- `crates/tau-tui/src/interactive/ui_tool_visibility_tests.rs`
- `specs/3609-mutating-transcript-breadcrumbs.md`

## Error Semantics
- Missing successful mutating target produces a system message that explicitly says no mutating target is available.
- Clipboard failures produce a visible system message with the underlying command failure.
- Mutating transcript breadcrumbs only render when the tool name is `write` or `edit` and detail is non-empty.

## Test Plan
- Add red tests for:
  - mutating transcript breadcrumb rendering
  - non-mutating transcript tool entries staying generic
  - `/copy-target` success path
  - `/copy-target` no-target failure path
- Run targeted `tau-tui` tests first, then full `cargo test -p tau-tui` and `cargo clippy -p tau-tui --all-targets --all-features -- -D warnings`.
