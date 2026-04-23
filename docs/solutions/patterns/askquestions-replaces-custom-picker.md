---
title: "Use Copilot's native askQuestions — do not build a custom picker extension"
category: patterns
date: 2026-04-19
tags: ["copilot", "vscode", "custom-agent", "billing", "premium-request", "askquestions", "extension"]
related: ["patterns/mcp-tool-namespacing-in-copilot.md"]
---
# Use Copilot's native askQuestions — do not build a custom picker extension

## Problem

To keep a custom Copilot agent's turn open (single premium request) across
multi-stage work, we wanted a "ask the user to pick the next stage" UI
that pauses the agent and returns the pick as structured tool-result
input. We tried two paths and both failed:

1. **MCP elicitation** (`elicitation/create` in the MCP spec). VS Code
   Copilot 0.44.1 does not implement the MCP client's elicitation
   capability — the tool call failed with "Request failed, retried with
   Autopilot" and the turn ended.

2. **Custom VS Code extension with `vscode.lm.registerTool` +
   `showQuickPick`.** We built a 120-line `gyre-ui` extension that
   registered `gyre_propose_next_stage` and blocked on a QuickPick.
   It worked technically, but the agent kept producing text-menu
   endings anyway because the extension wasn't installed on the user's
   machine — reinstall friction every time.

Meanwhile, the agent kept falling back to `"What's next? (a)... (b)..."`
text menus, ending the turn, and triggering a new premium request on
every user reply.

## Root cause

VS Code Copilot Chat shipped a **built-in `askQuestions` tool** in late
2025 / early 2026 ([vscode/issues/285952](https://github.com/microsoft/vscode/issues/285952),
[agent-tools docs](https://code.visualstudio.com/docs/copilot/agents/agent-tools)).
It is native, available without any extension, and:

- Pauses the agent turn with a visible "agent paused" indicator.
- Shows a labeled picker in chat.
- Returns the user's reply as structured tool-result input within the
  **same sendRequest** — no new premium request.
- Appears in the agent's tool list as the unqualified name `askQuestions`
  (no `mcp_*` prefix — it's an engine-provided tool, not MCP).

We wasted effort building a custom extension for a problem the platform
already solved. The extension was 120 lines of dead weight and required
a VSIX install step on every target machine.

## Solution

1. **Delete the custom extension** (`gyre-native/ui-extension/`). No
   install step, no VSIX to maintain.
2. **Add `askQuestions` to the agent's `tools:` frontmatter** (no prefix).
3. **In the agent prompt, mandate that Phase-4 / branching-decision
   responses end with an `askQuestions` call.** Schema:

   ```json
   {
     "name": "askQuestions",
     "arguments": {
       "questions": [
         {
           "question": "3 stages done. What's the next stage?",
           "options": [
             "Harden: run X against Y — ~5 min, low risk",
             "Deepen: cover gap Z — medium complexity",
             "Stop here — emit GYRE-TURN-COMPLETE"
           ]
         }
       ]
     }
   }
   ```

4. **Always include a "Stop here — emit GYRE-TURN-COMPLETE" option.**
   When the user's reply matches that text, treat as explicit stop.
5. **Forbid text-menu endings in the agent prompt.** The only legal
   response endings become: (A) a tool call that continues execution,
   (B) an `askQuestions` call, or (C) `GYRE-TURN-COMPLETE:` after an
   explicit stop.

Verification the loop now stays in one PR: the Copilot usage dashboard
should show 1 premium request per multi-stage Gyre session (covering
investigation → brainstorm → execute → reflect → next stage), not N.

## Prevention

- **Before building a custom VS Code extension for agent-UX needs,
  search the current Copilot Chat release notes for a native tool.**
  The platform is shipping new tools monthly — what required a plugin
  last month is built-in this month. `askQuestions` and `runSubagent`
  both landed in the Nov 2025 – Jan 2026 window.
- **Research the Copilot extension's tool inventory, not just its
  public API.** The tool picker's "Agent" / "Chat" groups list tools
  that aren't in the `vscode.*` API docs.
- **Default to native → MCP → custom-extension in that order.** Each
  step up the ladder adds install friction, version-skew surface, and
  maintenance cost.
- **When you catch yourself reinventing an obvious primitive** ("a
  blocking picker that returns structured input"), check the platform
  first. If it doesn't exist natively, check the community MCP
  registries before extension-land.
