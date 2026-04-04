---
title: "feat: Codex app-server WebSocket client"
type: feat
status: active
date: 2026-04-03
---

# feat: Codex app-server WebSocket client

## Overview

Replace the subprocess-based `CodexCliClient` with a WebSocket client that connects to `codex app-server --listen ws://IP:PORT`. This solves 5 critical problems: fake streaming, zombie processes, missing tool evidence, no session continuity, and timeout issues.

## Problem Statement

The current `CodexCliClient` (`codex_cli_client.rs`) spawns `codex exec` as a subprocess per request:

1. **Fake streaming** — `complete_with_stream()` calls `complete()` then fires `on_delta` once with the entire response (line 146-159). TUI shows zero progress for 1-5 minutes.
2. **Zombie processes** — even with `setpgid` + `killpg`, complex codex tasks spawn deep process trees that outlive the parent.
3. **No tool evidence** — codex runs tools (bash, write, etc.) internally. The gateway verifier never sees tool execution, enters retry loops, and times out.
4. **No session continuity** — each `codex exec` invocation is stateless. Context is re-sent every turn.
5. **Timeout issues** — complex tasks need 3-5 min. HTTP request timeouts expire first.

## Proposed Solution

A new `CodexAppServerClient` that implements `LlmClient` via the codex app-server WebSocket protocol.

### Protocol (verified via live testing)

```
1. initialize    → {clientInfo: {name: "tau", version: "0.1.0"}}
                 ← {codexHome, userAgent, ...}

2. thread/start  → {approvalPolicy: "never", sandbox: "workspace-write"}
                 ← {thread: {id: "..."}}

3. turn/start    → {threadId, input: [{type: "text", text: "..."}]}
                 ← {turn: {id: "..."}}

4. Stream events (notifications):
   - item/agentMessage/delta  → {delta: "...", itemId, threadId, turnId}
   - item/started             → tool execution started
   - item/completed           → tool execution completed
   - turn/completed           → {turn: {status: "completed"|"failed"|"interrupted"}}
   - thread/tokenUsage/updated → {tokenUsage: {inputTokens, outputTokens, ...}}
```

Key advantage: **threads persist across turns** — session continuity is built-in.

### Architecture

```
Gateway request
  ↓
CodexAppServerClient.complete_with_stream()
  ↓ (if no thread) → send thread/start
  ↓ send turn/start with user prompt
  ↓ read WS frames in loop:
  │   item/agentMessage/delta → fire on_delta handler (real streaming!)
  │   item/started/completed → track tool executions
  │   thread/tokenUsage/updated → accumulate usage
  │   turn/completed → break loop, return ChatResponse
  ↓
ChatResponse with:
  - message: assistant text from deltas
  - usage: real token counts from server
  - tool_calls: extracted from item/completed events
```

## Implementation Phases

### Phase 1: CodexAppServerClient core

**File: `crates/tau-provider/src/codex_appserver_client.rs`** (new)

```rust
pub struct CodexAppServerConfig {
    pub url: String,           // ws://127.0.0.1:PORT
    pub timeout_ms: u64,
    pub approval_policy: String,  // "never" for full-auto
    pub sandbox: String,          // "workspace-write"
}

pub struct CodexAppServerClient {
    config: CodexAppServerConfig,
    connection: Arc<Mutex<Option<CodexAppServerConnection>>>,
}

struct CodexAppServerConnection {
    ws: WebSocketStream<...>,
    thread_id: String,
    next_request_id: AtomicU64,
}
```

Implement:
- `connect_and_initialize()` — open WS, send `initialize`, receive response
- `ensure_thread()` — send `thread/start` if no thread_id, cache it
- `LlmClient::complete()` — delegates to `complete_with_stream(request, None)`
- `LlmClient::complete_with_stream()` — send `turn/start`, read events, fire deltas, return response

### Phase 2: Wire into provider selection

**File: `crates/tau-provider/src/client.rs`**

Add `build_openai_appserver_client()` parallel to `build_openai_codex_client()` (line 323). Gate on a new CLI flag `--openai-codex-appserver`.

When `--openai-codex-appserver` is set:
1. Spawn `codex app-server --listen ws://127.0.0.1:<port>` as a managed subprocess
2. Connect `CodexAppServerClient` to it
3. Return `Arc<dyn LlmClient>`

### Phase 3: App-server lifecycle management

**File: `crates/tau-provider/src/codex_appserver_lifecycle.rs`** (new)

Manage the codex app-server process:
- Start on first use (lazy)
- Health check via WS ping
- Restart on crash
- Kill on gateway shutdown
- Port selection: pick a free port or use configured one

### Phase 4: CLI flags

**File: `crates/tau-cli/src/cli_args.rs`**

New flags:
- `--openai-codex-appserver` (bool, default false) — use app-server instead of exec
- `--openai-codex-appserver-port` (u16, default 0 = auto) — WS port
- `--openai-codex-appserver-url` (string) — connect to existing app-server

## Technical Considerations

- **Dependencies**: `tokio-tungstenite` and `futures-util` already in workspace. Add to `tau-provider/Cargo.toml`.
- **Thread reuse**: Cache `thread_id` per session. The gateway already has `session_key` — map sessions to threads.
- **Error handling**: WS disconnect → reconnect and create new thread. Turn failures → return TauAiError.
- **Backpressure**: `on_delta` is synchronous. If the TUI can't keep up, deltas buffer in memory. This is fine — text deltas are small.
- **Security**: App-server listens on localhost only. No auth needed for loopback.
- **Compatibility**: `CodexCliClient` remains as fallback for users without app-server support.

## Acceptance Criteria

- [ ] `CodexAppServerClient` implements `LlmClient` trait
- [ ] Real streaming: `on_delta` fires incrementally as tokens arrive
- [ ] Session continuity: thread persists across multiple turns
- [ ] Token usage populated from `thread/tokenUsage/updated` events
- [ ] Tool execution events visible (item/started, item/completed)
- [ ] App-server process managed: start, health-check, restart, shutdown
- [ ] Complex tasks (game creation) complete successfully end-to-end
- [ ] `--openai-codex-appserver` CLI flag gates the new backend
- [ ] Existing `CodexCliClient` unchanged (fallback path)
- [ ] Tests: unit tests for WS message parsing, integration test with live app-server

## Files Changed

| File | Changes |
|------|---------|
| `crates/tau-provider/src/codex_appserver_client.rs` | NEW — WebSocket LlmClient implementation |
| `crates/tau-provider/src/codex_appserver_lifecycle.rs` | NEW — app-server process management |
| `crates/tau-provider/src/client.rs` | Add `build_openai_appserver_client()` factory |
| `crates/tau-provider/src/lib.rs` | Export new modules |
| `crates/tau-provider/Cargo.toml` | Add `tokio-tungstenite`, `futures-util` |
| `crates/tau-cli/src/cli_args.rs` | Add `--openai-codex-appserver` flags |

## Sources

- Codex app-server JSON schemas: `/tmp/codex-schema/v2/` (generated via `codex app-server generate-json-schema`)
- Protocol verified via live WebSocket test: `initialize` → `thread/start` → `turn/start` → deltas → `turn/completed`
- Existing `CodexCliClient`: `crates/tau-provider/src/codex_cli_client.rs`
- Provider factory: `crates/tau-provider/src/client.rs:323` (`build_openai_codex_client`)
- `LlmClient` trait: `crates/tau-ai/src/types.rs:363-376`
