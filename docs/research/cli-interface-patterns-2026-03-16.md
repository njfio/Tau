# CLI Interface Patterns - 2026-03-16

## Scope
Research notes for Issue `#3582` to align Tau TUI with strong terminal-agent interaction patterns, using OpenCode as the primary reference target.

## Sources
- OpenCode TUI docs: https://opencode.ai/docs/tui/
- OpenCode commands docs: https://opencode.ai/docs/commands/
- OpenCode permissions docs: https://opencode.ai/docs/permissions/
- OpenCode configuration docs: https://opencode.ai/docs/config/

## OpenCode Patterns That Matter

### 1. Transcript-first, not dashboard-first
OpenCode’s TUI is presented as the default working surface for interacting with the agent. The conversation is the product. Auxiliary state exists, but it does not dominate the screen.

Implication for Tau:
- The main pane must be the transcript.
- The current shell-header plus large operator-status dump is the wrong default.
- Secondary operational views should move into drawers, overlays, or toggles.

### 2. Command discoverability is built into the composer
OpenCode exposes slash commands and contextual affordances directly from the input model instead of making the user memorize hidden runtime commands. It also uses compact keyboard-first flows rather than full-screen mode changes.

Implication for Tau:
- The composer must advertise actions like interrupt, retry, session switch, details, approvals, and slash commands.
- We should stop treating the input area as a plain prompt line.

### 3. Details are contextual, not the primary layout
OpenCode has commands like `/details` and `/thinking`, plus other on-demand views, which indicates a principle: details exist, but are off to the side until requested.

Implication for Tau:
- Tools, memory, cortex, artifacts, and routines should live in a collapsible drawer or overlay.
- The default layout should not reserve major width for low-signal operational lists.

### 4. Sessions and sharing are first-class
OpenCode exposes `/sessions`, `/share`, and `/export`, which means run/session management is part of the operator UX rather than an afterthought.

Implication for Tau:
- Session key, session switching, replay/share/export, and saved runs should be visible from the TUI.
- This should not require switching to webchat or the filesystem.

### 5. Permission prompts are an interactive UX surface
OpenCode permissions are configured and surfaced as actual user decisions. This is critical because approvals are part of the product’s control loop.

Implication for Tau:
- Approvals/cancel/retry must render as an inline action strip or modal with explicit choices.
- Raw event text like `tool requires approval` is not acceptable as the main UX.

### 6. Customization exists, but the baseline matters more
OpenCode supports config, themes, and keybind customization. That matters, but the first-order lesson is that the default interaction model is already coherent.

Implication for Tau:
- We should fix the default layout before adding more knobs.
- Theme/keybind work is additive after transcript, composer, and detail surfaces are correct.

## Concrete Delta Between Tau And OpenCode

### Current Tau problems
- Starts like a shell/log console instead of an application.
- Uses too much width for side panels.
- Leaks infrastructure/status text into the primary user path.
- Has weak live-state affordances.
- Makes session/tool/detail flows feel bolted on.

### Target Tau behaviors
- Top line: compact context only.
- Center: transcript and live assistant/tool activity.
- Right drawer: optional details for tools, artifacts, memory, cortex, routines, sessions.
- Bottom composer: multiline input with visible actions and slash command discoverability.
- Inline action strips: approvals, retry, interrupt, continue.

## Design Constraints For Issue #3582
- Do not reintroduce dashboard-style panes as the default.
- Do not make the main experience depend on reading raw event logs.
- Do not hide core actions behind undocumented key combos.
- Do not let detail surfaces obscure the transcript by default.

## Immediate Implementation Consequences
1. Replace the current panel-first `interactive/ui.rs` layout with a transcript-first shell.
2. Add a compact top status line and a real composer/footer contract.
3. Move tools/events/memory/cortex/routines into a collapsible drawer.
4. Add PTY tests for resize, composer behavior, interrupt, retry, and detail toggles.
5. Align startup so `tau-unified.sh tui` lands directly in the new application view.
