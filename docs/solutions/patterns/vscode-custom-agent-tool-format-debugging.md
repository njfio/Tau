---
title: "VS Code .github/agents/ parser — authoritative format + diagnostic recipe"
category: patterns
date: 2026-04-21
tags: ["vscode", "copilot", "custom-agent", "mcp", "tool-allowlist", "state.vscdb", "debugging", "format-mismatch", "claude-code-format"]
related:
  - "patterns/mcp-tool-namespacing-in-copilot.md"
  - "patterns/askquestions-replaces-custom-picker.md"
  - "patterns/askquestions-non-stop-pick-completion-leak.md"
---

# VS Code `.github/agents/` parser — authoritative format + diagnostic recipe

## Problem

Custom agent file at `<workspace>/.github/agents/<name>.agent.md` appears correctly on disk, MCP server registers its tools successfully, tools show up in the chat tool picker with checkmarks — but the agent doesn't appear in Copilot Chat's mode selector and every tool call fails with "disabled in this chat" or just silent non-invocation.

Workspace `state.vscdb` → `chat.customModes` shows only the built-ins (Plan, Ask, Explore) with no entry for the custom agent. Alternatively the custom entry appears with `customTools: []` empty array. Window reloads don't help. Re-enabling tools in the picker doesn't help.

## Root cause (THE authoritative format reference)

**Copilot Chat 0.44.x uses two different agent parsers, and most online/community advice conflates them:**

1. **Built-in agent parser** — loads Plan, Ask, Explore from `$CODE_USER/globalStorage/github.copilot-chat/{plan,ask,explore}-agent/*.agent.md`. Uses fields like `tools: ['a', 'b']` inline-flow, `target: vscode`, `agents: [...]`, `handoffs: [...]`, `disable-model-invocation: true`.

2. **Claude-Code-style workspace parser** — scans `<workspace>/.github/agents/*.agent.md` AND `~/.claude/agents/*.agent.md`. Completely different format.

**The workspace parser's exact regex (from Copilot Chat `dist/extension.js`, `_parseAgentFile()`):**

```js
let c = a.match(/^name:\s*(.+)$/m),                             // REQUIRED
    l = a.match(/^description:\s*["']?([\s\S]*?)["']?$/m),      // optional
    u = a.match(/^model:\s*(.+)$/m),                            // REQUIRED
    d = a.match(/^allowedTools:\s*\n((?:\s+-\s+.+\n?)+)/m);     // bulleted

if (!c || !u) return;                                            // SILENT REJECTION
```

Key invariants:

- **`name:` is required.** Bare string value.
- **`model:` is required.** Bare string value. If absent, the parser returns `undefined` and the agent is silently rejected — no error surfaces anywhere the user can see.
- **`allowedTools:` is the tools field name** (NOT `tools:`).
- **`allowedTools:` must be bulleted YAML** (`  - toolname` indented lines), NOT inline-flow `[a, b, c]`. The regex only matches the bulleted form.
- **Field names ignored by this parser:** `target`, `kind`, `agents`, `handoffs`, `argument-hint`, `disable-model-invocation`, `visible`, `disabled`. These are Plan/Ask fields that don't apply here.

**Using Plan.agent.md as a reference template for a `.github/agents/` file is wrong** — Plan is loaded by a different parser and uses fields the workspace parser ignores. That was the trap that burned 5 debugging rounds in one session.

## Solution — correct `.github/agents/*.agent.md` template

```yaml
---
name: YourAgent
description: "One-line description."
model: claude-sonnet-4-5
allowedTools:
  - mcp_yourserver_tool1
  - mcp_yourserver_tool2
  - search
  - read
  - edit
  - web
  - vscode/askQuestions
  - agent/runSubagent
  - execute/runInTerminal
  - read/problems
---

[system prompt body here]
```

Rules:

- **MCP tool names** go in as `mcp_<server>_<tool>` (what the LLM emits in tool_call). The server name is the key under `servers:` in `.vscode/mcp.json`, not the `name` field inside the MCP server's `register*` call.
- **Native Copilot tools** go in with either bare category names (`search`, `read`, `edit`, `web`) or `group/name` form (`vscode/askQuestions`, `execute/runInTerminal`, `read/problems`).
- **Subagents** — use `agent/runSubagent` to allow the agent to dispatch sub-tasks.
- **`model:`** — any string value works for the parser's regex. Whether that model is actually available at runtime is a separate concern (VS Code tries to resolve it against user's Copilot chat-model picker). Safe defaults: `claude-sonnet-4-5`, `gpt-5`, or the current Copilot default.

## Diagnostic recipe (in priority order)

If your custom agent isn't appearing in the chat mode selector, work through these steps. **Do them in order — each one eliminates a specific failure layer.**

### 1. Verify the parser's exact regex matches your file

Don't rely on visual inspection or generic YAML validators. Use the parser's own regex:

```bash
python3 -c "
import re
text = open('<your-workspace>/.github/agents/<name>.agent.md').read()
m = re.match(r'^---\n([\s\S]*?)\n---\n([\s\S]*)$', text)
if not m: print('FAIL: no frontmatter block')
else:
    fm = m.group(1)
    name_ok = bool(re.search(r'^name:\s*(.+)\$', fm, re.M))
    model_ok = bool(re.search(r'^model:\s*(.+)\$', fm, re.M))
    tools_m = re.search(r'^allowedTools:\s*\n((?:\s+-\s+.+\n?)+)', fm, re.M)
    tools_ct = len(re.findall(r'^\s+-\s+', tools_m.group(1) if tools_m else '', re.M))
    print(f'name={name_ok} model={model_ok} allowedTools={tools_ct}')
    if not name_ok or not model_ok:
        print('PARSER WILL REJECT THIS FILE')
"
```

Expected output: `name=True model=True allowedTools=<N>`. Anything else = file won't parse.

### 2. Check the workspace DB for the cached registration

```bash
WS_HASH=$(find ~/Library/Application\ Support/Code/User/workspaceStorage \
  -name workspace.json -exec grep -l "$(pwd)" {} \; | head -1 | xargs dirname | xargs basename)
DB=~/Library/Application\ Support/Code/User/workspaceStorage/$WS_HASH/state.vscdb
sqlite3 "$DB" "SELECT value FROM ItemTable WHERE key='chat.customModes';" | python3 -c "
import json, sys
for m in json.loads(sys.stdin.read()):
    print(f\"{m['name']}: customTools={len(m.get('customTools',[]))}\")
"
```

**Interpretations:**

- `YourAgent: customTools=<N>` where N matches your `allowedTools` count → success
- Agent missing from the list → parser rejected the file (finding #1 should catch this first)
- `YourAgent: customTools=0` → parse succeeded but every tool name was unrecognized by VS Code's internal registry → probably wrong tool name convention (check if MCP tool names use the `mcp_<server>_<tool>` prefix form)

### 3. Grep the extension itself for authoritative behavior

When in doubt, read the parser's source. The compiled JS is ugly but greppable:

```bash
EXT=/Users/n/.vscode/extensions/github.copilot-chat-*/dist/extension.js
grep -oE '_parseAgentFile[^{]*\{[^}]{200,1000}' $EXT | head -100
```

Look for the exact regex patterns, the `if (!X || !Y) return` early-rejections, and the shape of the returned config object.

### 4. Check extension conflicts

Old extensions registering chat participants with the same name will block the custom-agent from registering under that name:

```bash
ls ~/.vscode/extensions/ | grep -iE 'yourname|gyre'
for d in ~/.vscode/extensions/<your-extension-name>-*; do
  python3 -c "import json; p=json.load(open('$d/package.json')); print('chatParticipants:', p.get('contributes',{}).get('chatParticipants'))"
done
```

If any installed extension contributes a `chatParticipants` entry with your agent's name, uninstall it: `rm -rf ~/.vscode/extensions/<extension-name>-*`.

### 5. Force cache invalidation + reload

After any format fix, clear the stale customModes entry before reload so VS Code re-parses fresh:

```bash
python3 <<'PY'
import json, sqlite3
db = "<path-to-state.vscdb>"
c = sqlite3.connect(db)
row = c.execute("SELECT value FROM ItemTable WHERE key='chat.customModes'").fetchone()
modes = [m for m in json.loads(row[0]) if m.get('name','') != 'YourAgent']
c.execute("UPDATE ItemTable SET value=? WHERE key='chat.customModes'", (json.dumps(modes),))
c.commit()
PY
```

Then `Cmd-Shift-P` → "Developer: Reload Window" → open Copilot Chat → send any message to trigger discovery → re-run step 2 to verify.

## Prevention

1. **Never use Plan.agent.md or Ask.agent.md as a format reference for `.github/agents/` files.** They're for a different parser. The Copilot Chat 0.44 workspace parser is Claude-Code-style.

2. **If you build an install kit that drops agent files**, include a format-validation check as a pre-commit or pre-ship step that runs the parser's actual regex against every shipped agent.md. The install-kit-sync check is a good place to add this — one Python snippet alongside the content diff.

3. **When debugging a silent rejection, grep the extension source first.** A compiled JS bundle is uglier than docs, but it's the authoritative source. Docs lag; code ships.

4. **Multiple parser systems coexist in Copilot Chat.** If you see one custom agent working and another failing with the "same" format, they might be loaded by different parsers. Check: where does the working one live on disk? `globalStorage/github.copilot-chat/...` vs `<workspace>/.github/agents/`? Those are different loaders.

## Historical arc (what happened the first time)

One Gyre session, five debugging rounds, each targeting the wrong layer:

1. **YAML corruption by normalizer** — fixed by switching to inline-flow YAML.
2. **Wrong naming convention** — switched from `mcp_gyre_*` to `gyre/*`, then back.
3. **Schema field mismatch in handlers** (camelCase vs snake_case) — fixed with aliases.
4. **Added `target: vscode`** — didn't help because this parser doesn't use `target:`.
5. **Uninstalled conflicting old extension** — necessary but not sufficient.

The actual fix came from grepping `_parseAgentFile` in the extension code and discovering the Claude-Code-style format. **One `grep` on round 1 would have saved the other four rounds.**

Total time lost to wrong-layer debugging in that session: ~2 hours. This compound doc exists so the next session hits round 1 and stops.
