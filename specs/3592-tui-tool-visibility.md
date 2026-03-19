# 3592 TUI Tool Visibility

## Objective
Make live and recent tool execution clearly visible from the main TUI shell so users can tell when Tau is acting, what tool is running, and whether the last tool succeeded or failed without relying on the side tools panel.

## Inputs/Outputs
- Input: interactive TUI app state containing tool entries and agent state
- Output: main-shell render that exposes running and recent tool execution state in the chat/transcript area and supporting tests proving the visible behavior

## Boundaries/Non-goals
- Do not change runtime or provider tool execution semantics.
- Do not redesign the full TUI layout.
- Do not change webchat in this issue.
- Keep behavior changes limited to visibility of tool execution and related render state.

## Failure modes
- A running tool remains invisible unless the tools panel is open.
- The main shell does not distinguish successful and failed recent tool execution.
- Render changes regress the existing side tools panel.
- The real interactive render path lacks test coverage for the visible tool state.

## Acceptance criteria
- [ ] When a tool is running, the main shell renders a visible live tool activity summary.
- [ ] When the latest tool completes, the main shell renders the tool name and a distinct success or failure status.
- [ ] Failed tool activity is visually distinct from successful tool activity.
- [ ] The side tools panel continues to render tool history.
- [ ] At least one integration-style TUI test proves the real shell render includes visible tool activity.

## Files to touch
- `specs/3592-tui-tool-visibility.md`
- `crates/tau-tui/src/interactive/tools.rs`
- `crates/tau-tui/src/interactive/ui.rs`
- `crates/tau-tui/src/interactive/ui_*.rs`
- `crates/tau-tui/src/interactive/mod.rs`
- `crates/tau-tui/src/interactive/*test*`

## Error semantics
- Fail loud through tests if tool visibility is absent from the main shell.
- Do not add silent fallback rendering paths that hide missing tool state.

## Test plan
- Red: add render tests proving running tool state and last completed tool state are absent/present in the main shell as specified.
- Green: implement a main-shell tool visibility component and wire it into the interactive renderer.
- Refactor: split oversized interactive UI modules into smaller renderer files while preserving test coverage.
- Integration: run the full tau-tui test suite and at least one interactive render-focused test exercising the real app state path.
