# Objective
Make the interactive TUI respond to mouse wheel scrolling and left-click focus/highlighting so users can scroll overflowing panels and visibly focus the panel under the cursor.

# Inputs/Outputs
- Input: crossterm `Event::Mouse` events during the interactive TUI event loop.
- Output: the hovered/clicked panel becomes the active focus target, mouse wheel events scroll the correct panel, and the focused panel border/highlight updates in the rendered UI.

# Boundaries/Non-goals
- Do not implement terminal text selection or clipboard copy with the mouse.
- Do not redesign the overall TUI layout.
- Do not change model/runtime/provider behavior.
- Do not add silent fallbacks when mouse events cannot be mapped to a panel.

# Failure modes
- Mouse wheel over a scrollable panel changes nothing: fail tests.
- Left-click over chat/tools does not change focus styling: fail tests.
- Mouse events over non-panel areas panic or corrupt state: fail tests.
- Keyboard navigation regresses after mouse support is added: fail tests.

# Acceptance criteria
- [ ] Mouse wheel scrolling over the chat panel scrolls chat content.
- [ ] Mouse wheel scrolling over the tools panel scrolls tool history when the tools panel is visible.
- [ ] Left-clicking chat or tools updates `focus` so the clicked panel is visibly highlighted.
- [ ] Mouse events outside interactive panels do not panic and do not corrupt app state.
- [ ] Existing keyboard focus/scroll controls still work.
- [ ] At least one integration test exercises the real ratatui/crossterm mouse path through the interactive app state.

# Files to touch
- `crates/tau-tui/src/interactive/app.rs`
- `crates/tau-tui/src/interactive/mod.rs`
- `crates/tau-tui/src/interactive/ui.rs`
- `crates/tau-tui/src/interactive/ui_body.rs`
- `crates/tau-tui/src/interactive/ui_tools.rs`
- `crates/tau-tui/src/interactive/chat.rs`
- `crates/tau-tui/src/interactive/tools.rs`
- New focused modules for mouse/layout handling and tests if needed
- `specs/3596-tui-mouse-interactions.md`

# Error semantics
- Mouse handlers must ignore unmapped coordinates without panicking.
- No silent fallback to unrelated panels when hit-testing fails.
- Rendering/layout helpers must keep a single source of truth for panel rectangles.

# Test plan
- Add red tests for mouse wheel scrolling in chat.
- Add red tests for mouse click focus/highlight in chat/tools.
- Add a red integration test that sends real mouse events through the interactive app/event path.
- Run `cargo test -p tau-tui -- --nocapture` after implementation and refactor.
