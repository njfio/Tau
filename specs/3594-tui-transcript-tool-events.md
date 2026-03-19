# Objective
Render live tool execution and final tool outcomes directly in the interactive TUI transcript so the conversation history shows that Tau acted, not just that a final assistant message appeared.

# Inputs/Outputs
- Input: `App` state with `chat` messages and `tools` entries in `crates/tau-tui/src/interactive`.
- Output: The chat panel render includes explicit transcript-visible tool event lines/cards while tools run and after they finish.

# Boundaries/Non-goals
- Do not change runtime or provider tool execution semantics.
- Do not remove or redesign the existing side tools panel.
- Do not redesign the entire transcript layout beyond what is needed to make tool activity visible.
- Do not persist synthetic tool breadcrumbs into runtime session storage.

# Failure modes
- No tool entries exist: transcript renders only normal chat history.
- A tool is running but the transcript omits it: fail tests.
- A failed or timed out tool completes but the transcript omits it: fail tests.
- Tool detail text is empty or missing: renderer still shows tool name and status without panicking.

# Acceptance criteria
- [ ] When `ToolPanel` has a running tool entry, the chat panel renders a transcript-visible tool activity block containing the tool name and running status.
- [ ] When the latest tool entry is successful, failed, or timed out, the chat panel renders a transcript-visible tool result block containing the tool name and terminal status.
- [ ] Tool transcript blocks render in the main chat history area, not only in the summary strip or side tools panel.
- [ ] Existing tool summary strip and side tools panel behavior remain intact.
- [ ] At least one integration test exercises the real `render_chat_panel` path with both transcript tool blocks and the side tools panel present.

# Files to touch
- `crates/tau-tui/src/interactive/ui_chat.rs`
- `crates/tau-tui/src/interactive/ui_chat_tool_lines.rs`
- `crates/tau-tui/src/interactive/ui_tool_visibility_tests.rs`
- `specs/3594-tui-transcript-tool-events.md`

# Integration points
- `crates/tau-tui/src/interactive/ui.rs` calls `render_chat_panel`.
- `crates/tau-tui/src/interactive/ui_chat.rs` appends transcript tool lines from the live `ToolPanel`.
- `crates/tau-tui/src/interactive/ui_tool_visibility_tests.rs` exercises the real ratatui render path with both transcript and side-panel visibility assertions.

# Error semantics
- Rendering code must not silently panic on empty tool details.
- This is a pure UI slice; failures are test failures, not runtime fallbacks.
- No new silent fallbacks are allowed.

# Test plan
- Add a red test for running tool activity rendered inside the transcript area.
- Add a red test for terminal tool result rendered inside the transcript area.
- Add an integration render test that asserts both transcript tool visibility and side tools panel visibility through the real render path.
