---
title: "fix: Gateway transport tool usage — model outputs text instead of calling tools"
type: fix
status: active
date: 2026-03-25
---

# fix: Gateway Transport Tool Usage

## Overview

When using `transport=gateway` with the TUI, the model (gpt-5.3-codex) outputs code as text to the screen instead of using the registered tools (bash, write, edit) to create files. The TUI shows "Tools (0 active / 0 total)" despite tools being properly registered server-side.

## Problem Statement

**Symptom:** User asks "create a tetris game using phaserjs" — model outputs HTML/JS code as text in the chat instead of calling `bash` or `write` tools to create files on disk.

**Root Cause Analysis (confirmed through code tracing):**

The tool registration and request pipeline is **correct**:
1. `startup_transport_modes.rs:492` — `GatewayToolRegistrarFn` calls `register_builtin_tools()` (28 tools including bash, read, write, edit, grep, glob)
2. `openresponses_execution_handler.rs:54` — `state.config.tool_registrar.register(&mut agent)` runs on every request
3. `lib.rs:2608-2627` — Agent builds `ChatRequest` with all tool definitions and `tool_choice: Auto`
4. LLM receives the full tool list

**The model receives tools but chooses not to call them.** This is likely because:

- The system prompt may not instruct the model strongly enough to use tools
- The `gpt-5.3-codex` model doesn't have a catalog entry (`model catalog warning: missing entry for 'openai/gpt-5.3-codex'`), so capability guardrails are skipped
- The TUI displays "Tools (0 active / 0 total)" because it tracks tools locally, not from the gateway — this is a **display bug**, not a tool availability bug

## Proposed Solution

Three fixes, ordered by impact:

### Fix 1: Strengthen System Prompt Tool Instructions (High Impact)

The gateway system prompt needs explicit instructions telling the model to use tools for file operations.

**Where:** The system prompt resolved at `openresponses_execution_handler.rs:35` via `state.resolved_system_prompt()`.

**What to add:**
```
## Tool Usage Instructions
You have access to tools for file I/O, shell commands, and memory. ALWAYS use tools to:
- Create files: use the `write` tool
- Edit files: use the `edit` tool
- Run commands: use the `bash` tool
- Search files: use `grep` or `glob` tools
NEVER output code as text when you can use a tool to create/modify files directly.
```

**Where to inject:** In `crates/tau-onboarding/src/startup_prompt_composition.rs` or in the `.tau/SOUL.md` file.

### Fix 2: Add gpt-5.3-codex to Model Catalog (Medium Impact)

**Where:** `crates/tau-provider/src/model_catalog.rs` (or wherever `ModelCatalog::built_in()` is defined)

**What:** Add an entry for `openai/gpt-5.3-codex` with appropriate capabilities (tool_use: true, etc.). The missing catalog entry means capability guardrails are skipped, which may affect how the request is constructed.

### Fix 3: Fix TUI "Tools (0 active / 0 total)" Display (Low Impact, UX)

**Where:** `crates/tau-tui/src/interactive/status.rs` and `app.rs`

**What:** The TUI currently shows 0 tools because it calculates tools locally instead of querying the gateway. After a successful response that includes tool calls, update the tool count display. Alternatively, query the gateway's `/gateway/tools` endpoint on startup.

## Technical Considerations

- Fix 1 is the most impactful — model behavior is primarily driven by system prompt instructions
- Fix 2 ensures the model is recognized and appropriate features are enabled
- Fix 3 is cosmetic but reduces user confusion
- The `.tau/SOUL.md` file already exists and is included in the system prompt — this is the natural place for tool usage instructions

## Acceptance Criteria

- [ ] Model uses `bash` or `write` tool when asked to create a file (not text output)
- [ ] TUI shows non-zero tool count after tool-using responses
- [ ] `gpt-5.3-codex` has a model catalog entry with tool_use capability
- [ ] System prompt includes explicit tool usage instructions

## Sources & References

- Tool registration: `crates/tau-tools/src/tools/registry_core.rs:570`
- Gateway tool registrar setup: `crates/tau-onboarding/src/startup_transport_modes.rs:492`
- Agent request construction: `crates/tau-agent-core/src/lib.rs:2608-2627`
- Gateway execution handler: `crates/tau-gateway/src/gateway_openresponses/openresponses_execution_handler.rs:36-54`
- TUI tool display: `crates/tau-tui/src/interactive/status.rs`
- Model catalog warning: gateway startup log
