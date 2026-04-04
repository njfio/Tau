---
title: "fix: TUI scroll, copy, and auto-follow UX overhaul"
type: fix
status: active
date: 2026-04-03
---

# fix: TUI scroll, copy, and auto-follow UX overhaul

## Overview

The tau-tui interactive mode has three core UX problems: broken scrolling (message-index-based instead of line-based), no text copy mechanism, and fragile auto-follow. This plan fixes all three with minimal architectural change.

## Problem Statement

1. **Scroll is broken.** `ChatPanel.scroll_offset` (chat.rs:34) is a message index. `compute_chat_scroll` (ui_chat.rs:135) maps it to rendered lines via `msg_idx * 3` — a hardcoded approximation that fails for multi-line messages. A 50-line agent response scrolls the same amount as a 1-line "ok".

2. **Can't copy text.** `EnableMouseCapture` blocks native terminal selection. The only clipboard feature is `/copy-target` which copies file paths, not message content. No keybinding exists. Uses `pbcopy` only (no Linux/Windows).

3. **Auto-follow is fragile.** `push_timestamped_message()` calls `scroll_to_bottom()`, but any manual `scroll_up` permanently breaks auto-follow because there's no follow-mode flag.

## Proposed Solution

Seven fixes in priority order, implemented as a single PR across 6 files.

### Phase 1: Fix scroll model (highest impact)

**Task 1.1 — Line-based scroll offset** (`chat.rs`, `ui_chat.rs`)

Change `scroll_offset` from message-index to line-offset semantics:

```rust
// chat.rs — ChatPanel
pub struct ChatPanel {
    messages: Vec<ChatMessage>,
    scroll_offset: usize,    // NOW: lines from top (was: message index)
    max_scroll: usize,       // NEW: set during render
    follow_mode: bool,       // NEW: auto-follow flag
    max_messages: usize,
}
```

Update scroll API:
- `scroll_up(n)` — `scroll_offset = scroll_offset.saturating_sub(n); follow_mode = false;`
- `scroll_down(n)` — `scroll_offset = (scroll_offset + n).min(max_scroll);` re-engage follow if at bottom
- `scroll_to_bottom()` — `scroll_offset = max_scroll; follow_mode = true;`
- `set_max_scroll(n)` — called from render to update max

Replace `compute_chat_scroll` in `ui_chat.rs`:
```rust
fn compute_chat_scroll(app: &App, total_lines: usize, visible_height: usize) -> u16 {
    if total_lines <= visible_height { return 0; }
    let max = total_lines.saturating_sub(visible_height);
    if app.chat.follow_mode() {
        return max as u16;
    }
    app.chat.scroll_offset().min(max) as u16
}
```

**Task 1.2 — Mouse scroll delta 1→3** (`app_mouse.rs`)

Change `scroll_panel_at_cursor` calls from delta `1` to `3` (standard terminal convention of 3 lines per wheel tick).

**Task 1.3 — Auto-follow mode** (`chat.rs`, `app.rs`)

- Add `follow_mode: bool` to `ChatPanel`, default `true`
- `scroll_up()` sets `follow_mode = false`
- `scroll_to_bottom()` sets `follow_mode = true`
- `scroll_down()` re-engages follow when `scroll_offset >= max_scroll`
- `push_timestamped_message()` only calls `scroll_to_bottom()` when `follow_mode` is true

### Phase 2: Fix copy/clipboard

**Task 2.1 — Document modifier-key selection** (`app_keys.rs` help overlay)

Add to the help text rendered by `?`: `"Shift+drag or Option+drag to select text (terminal native)"`.

**Task 2.2 — OSC 52 clipboard** (`app_copy_target.rs`)

Add `osc52_copy(text: &str)` function using the OSC 52 escape sequence:
```rust
fn osc52_copy(text: &str) {
    let encoded = base64::engine::general_purpose::STANDARD.encode(text);
    let _ = std::io::stdout().write_all(format!("\x1b]52;c;{encoded}\x07").as_bytes());
    let _ = std::io::stdout().flush();
}
```

Update `copy_to_clipboard()` to try OSC 52 first, fall back to platform command.

**Task 2.3 — `/copy` and `/copy-last` commands** (`app.rs` command handling)

- `/copy` — copy full chat transcript to clipboard via OSC 52
- `/copy-last` — copy the last assistant message content
- `y` in normal mode with chat focused — yank last assistant response

**Task 2.4 — Toggle mouse capture** (`app.rs`, `app_keys.rs`)

- `/toggle-mouse` command or `Ctrl-M` in normal mode
- Calls `DisableMouseCapture` / `EnableMouseCapture` via crossterm
- Shows `[MOUSE OFF]` in status bar when disabled

## Technical Considerations

- **No new dependencies required.** OSC 52 is raw escape sequences. `base64` is already in the dependency tree via other crates.
- **Backward compatible.** All changes are additive. Default behavior (follow mode on, mouse capture on) matches current behavior.
- **Test strategy:** Use existing `TestBackend` pattern. Test scroll math with known message heights. Test auto-follow state transitions. Mouse/clipboard are untestable in unit tests but verifiable manually.
- **Performance:** `compute_chat_scroll` is called every render tick (100ms). The new version is O(1) — just reads `scroll_offset` and clamps. No regression.

## Acceptance Criteria

- [ ] Scroll offset tracks rendered lines, not message indices
- [ ] Mouse wheel scrolls 3 lines per tick
- [ ] Auto-follow sticks to bottom on new messages
- [ ] Manual scroll-up disengages auto-follow
- [ ] `G` key and scroll-to-bottom re-engage auto-follow
- [ ] Help overlay documents modifier-key text selection
- [ ] `/copy-last` copies last assistant message via OSC 52
- [ ] `/copy` copies full transcript
- [ ] `y` in normal mode yanks last assistant response
- [ ] `/toggle-mouse` disables mouse capture for native selection
- [ ] All existing tests pass
- [ ] New tests for scroll math and auto-follow state transitions

## Files Changed

| File | Changes |
|------|---------|
| `crates/tau-tui/src/interactive/chat.rs` | Add `follow_mode`, `max_scroll`; change scroll semantics to line-based |
| `crates/tau-tui/src/interactive/ui_chat.rs` | Replace `compute_chat_scroll` with line-based version; call `set_max_scroll` during render |
| `crates/tau-tui/src/interactive/app_mouse.rs` | Change scroll delta from 1 to 3 |
| `crates/tau-tui/src/interactive/app.rs` | Update `push_timestamped_message` for follow_mode; add `/copy`, `/copy-last`, `/toggle-mouse` commands |
| `crates/tau-tui/src/interactive/app_keys.rs` | Add `y` yank binding, `Ctrl-M` toggle mouse, update help text |
| `crates/tau-tui/src/interactive/app_copy_target.rs` | Add `osc52_copy`, update `copy_to_clipboard` fallback chain |

## Sources

- Ratatui `Paragraph::scroll()` takes `(u16, u16)` line offset — confirms line-based is correct
- OSC 52 spec: supported by iTerm2, Alacritty, WezTerm, Ghostty, Kitty, tmux
- Crossterm `EnableMouseCapture`/`DisableMouseCapture` — toggle is straightforward
- Existing test pattern: `app_mouse_tests.rs` uses direct `handle_mouse()` + assert on state
