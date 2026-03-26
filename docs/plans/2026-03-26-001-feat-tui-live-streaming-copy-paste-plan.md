---
title: "feat: TUI live token/cost counters, thinking stream, copy/paste"
type: feat
status: active
date: 2026-03-26
---

# feat: TUI Live Streaming, Thinking Output, Copy/Paste

## Overview

Three focused TUI improvements: (1) live token/cost updates during streaming, (2) visible thinking/reasoning output, (3) mouse scroll and clipboard copy.

## Implementation Units

### Unit 1: Live Token/Cost SSE Events

**Goal:** Status bar updates tokens and cost during streaming, not just after completion.

**Files:**
- `crates/tau-gateway/src/gateway_openresponses/openresponses_execution_handler.rs` — emit new SSE events
- `crates/tau-tui/src/interactive/gateway_client.rs` — parse new events
- `crates/tau-tui/src/interactive/app.rs` — update status bar on events

**Approach:**
1. In the gateway execution handler, subscribe to `AgentEvent::TurnEnd` and `AgentEvent::CostUpdated`. When received and `stream_sender` is active, emit:
   - `response.usage.delta` with `{input_tokens, output_tokens, total_tokens}`
   - `response.cost.delta` with `{turn_cost_usd, cumulative_cost_usd}`
2. In TUI `gateway_client.rs`, add `GatewayStreamEvent::UsageUpdate` and `CostUpdate` variants
3. In TUI `app.rs` tick(), handle these events to update `status.total_tokens` and `status.total_cost_cents`

**Verification:** During a multi-turn response, the status bar token count increments after each turn, not just at the end.

---

### Unit 2: Thinking/Reasoning Stream

**Goal:** Show model thinking output in the TUI chat, not just "THINKING" label.

**Files:**
- `crates/tau-gateway/src/gateway_openresponses/openresponses_execution_handler.rs` — emit thinking events
- `crates/tau-tui/src/interactive/gateway_client.rs` — parse thinking events
- `crates/tau-tui/src/interactive/app.rs` — render thinking text
- `crates/tau-tui/src/interactive/chat.rs` — optional: thinking role styling

**Approach:**
1. In gateway: subscribe to `AgentEvent::MessageAdded`. If the message role is `Assistant` and the turn hasn't produced output_text.delta yet, emit `response.thinking.delta` SSE event. Also check if the codex CLI stdout contains thinking markers.
2. In TUI gateway_client: add `GatewayStreamEvent::Thinking(String)` variant for thinking deltas
3. In TUI app.rs: when a Thinking event arrives and status is Thinking, create/update a dimmed "thinking" message in chat. When the first real Delta arrives, finalize the thinking message.
4. In chat.rs: add `MessageRole::Thinking` with distinct styling (dimmed gray, italic)

**Verification:** When the model thinks before responding, the TUI shows the reasoning text as it streams (dimmed), then the actual response appears normally.

---

### Unit 3: Mouse Scroll + Clipboard Copy

**Goal:** Users can scroll with mouse wheel and copy text with keyboard shortcut.

**Files:**
- `crates/tau-tui/src/interactive/app_keys.rs` — add copy keybinding
- `crates/tau-tui/src/interactive/app.rs` — handle mouse events, add clipboard support
- `crates/tau-tui/src/interactive/main_loop.rs` or wherever the event loop lives — enable mouse capture
- `Cargo.toml` for tau-tui — add `arboard` or `cli-clipboard` dependency

**Approach:**
1. Enable mouse capture in crossterm terminal setup
2. Route `MouseEvent::ScrollUp/Down` to the focused panel's scroll method
3. In normal mode, `y` copies the last assistant message to clipboard
4. `Y` copies all messages to clipboard
5. Use `arboard` crate for cross-platform clipboard access

**Verification:** Mouse wheel scrolls the chat. Pressing `y` in normal mode copies the last response to clipboard (pasteable in another app).

## Acceptance Criteria

- [ ] Token count in status bar updates during streaming (after each agent turn)
- [ ] Cost updates during streaming
- [ ] Thinking/reasoning text visible in chat as dimmed text during response generation
- [ ] Mouse wheel scrolls focused panel
- [ ] `y` in normal mode copies last assistant message to system clipboard
- [ ] No regression in existing TUI functionality
