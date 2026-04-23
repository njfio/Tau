---
title: "MCP tool names are namespaced mcp_<server>_<tool> in Copilot Chat"
category: patterns
date: 2026-04-19
tags: ["mcp", "copilot", "vscode", "tool-picker", "custom-agent"]
related: ["patterns/askquestions-replaces-custom-picker.md"]
---
# MCP tool names are namespaced mcp_<server>_<tool> in Copilot Chat

## Problem

Gyre's `.github/agents/gyre.agent.md` listed backlog tools in its `tools:`
frontmatter as `backlog_read`, `backlog_next`, `backlog_replace`, etc. In
VS Code Copilot Chat 0.44.x, these references did not bind — Copilot either
fell back to built-in tools or failed silently. When the user manually
inspected the chat tool picker, one tool was shown as explicitly disabled:

```
Tool mcp_gyre_backlog_replace is currently disabled by the user,
and cannot be called.
```

The agent then produced text-menu "What's next? (a)... (b)..." endings
because it couldn't reach the tools it needed to continue the loop —
ending the Copilot turn and burning an extra premium request each time.

## Root cause

VS Code Copilot Chat exposes MCP-server-provided tools to agents under a
namespaced name: **`mcp_<server-name>_<tool-name>`**. The `<server-name>`
is the key in `.vscode/mcp.json` (e.g. `"gyre"` under `"servers"`), not
the MCP server's self-reported `name` field. So tools registered by the
server as `backlog_read` surface to Copilot as `mcp_gyre_backlog_read`.

Two consequences:

1. A custom-agent `tools:` list must use the namespaced form or the
   reference fails to bind. The agent runs without those tools.
2. The chat tool picker shows namespaced names. Users disabling a tool
   in the picker disables the namespaced entry. There is no programmatic
   way to override a user's picker preference — it's a per-thread UI
   state and the only fix is the visible checkbox.

## Solution

1. In the agent manifest frontmatter, reference MCP tools with the
   namespaced name:

   ```yaml
   tools:
     - mcp_gyre_backlog_read
     - mcp_gyre_backlog_next
     - mcp_gyre_backlog_replace
     # ... not backlog_read
   ```

2. In the agent's prose prompt, also reference the namespaced name so the
   model produces correct tool calls:

   ```md
   Your first tool call must be `mcp_gyre_backlog_read`.
   ```

3. Document for users that on a fresh chat they must open the tool picker
   (🔧 icon above chat input) and enable every `mcp_<server>_*` entry
   their agent depends on. `install.sh` cannot do this — it's UI state.

4. Add belt-and-suspenders: when an MCP tool detects drain (e.g. the
   Gyre MCP server's `backlog_mark_done` notices no pending deliverables
   remain), include in the tool's response text a forcing instruction
   naming the *next tool the agent must call by its namespaced name*.
   The model sees the tool result and has no ambiguity about what to do.

## Prevention

- **Never reference MCP tools by their unqualified server-side name in
  agent manifests or prompts.** Always use `mcp_<server>_<tool>`.
- Add a pre-push lint rule on agent manifests: `grep -v mcp_` over the
  tool list should catch any unqualified MCP-tool references.
- When onboarding a new user, include in the install README: *"open
  the tool picker and enable all tools in the <server> group before
  first run."*
- When designing an MCP server's tool-result text, have drain-path tools
  embed the next expected tool call by its full namespaced name — the
  agent's recovery path becomes deterministic instead of prompt-reliant.
