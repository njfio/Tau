# Spec: Issue #3582 - Redesign Tau TUI as a Transcript-First Operator Terminal

Issue: https://github.com/njfio/Tau/issues/3582
Status: Research-updated

## Objective
Redesign Tau TUI from a panel-heavy operator/debug shell into a transcript-first terminal agent interface closer to OpenCode, Claude Code, and Codex CLI. The result must feel like an application instead of a log console: conversation first, compact context always visible, details on demand, and live activity/interrupt/approval flows that do not collapse into raw runtime text.

## Inputs/Outputs
- Inputs:
  - Shared operator turn/task state protocol from Issue #3581.
  - Local TUI command input and keyboard shortcuts.
  - Runtime approval, tool, artifact, memory, cortex, and job state updates.
  - OpenCode interaction patterns for transcript-first navigation, command discoverability, detail toggles, session access, and permission prompts.
- Outputs:
  - A transcript-first TUI layout with a compact top bar, conversation stream, right-side detail drawer, and a fixed composer.
  - Keyboard-driven command surface with slash commands, session access, detail/thinking toggles, interrupt/retry, and approval controls.
  - A layout contract that degrades gracefully on smaller terminals while preserving transcript readability and composer usability.

## Boundaries / Non-goals
- No webchat redesign in this issue.
- No independent UI-only event model; TUI must consume the shared state contract.
- No swarm orchestration implementation beyond terminal hooks and navigation affordances.
- No fallback to panel-dump/dashboard behavior as the primary interaction mode.
- No speculative terminal chrome unrelated to operator workflows.

## Failure Modes
1. Long-running turn with tools and partial assistant output.
   - Expected: transcript remains primary, activity is visible inline, and detail drawers can be opened without displacing the conversation.
2. Runtime/provider failure during an active turn.
   - Expected: failure is rendered inline with retry/interrupt context and the TUI does not exit.
3. Small terminal size or resize during active work.
   - Expected: layout collapses secondary surfaces first, keeps composer visible, and preserves transcript scroll position.
4. Approval-required turn.
   - Expected: approval state is visually distinct, keyboard accessible, and represented as an action strip rather than generic event spam.
5. Operator wants tools, memory, cortex, artifacts, or sessions.
   - Expected: a focused secondary drawer/overlay is available on demand and is collapsed by default.

## Acceptance Criteria (testable booleans)
- [ ] AC-1: TUI default layout is transcript-first, with the conversation stream occupying the primary visual space and no persistent dashboard-style panel grid.
- [ ] AC-2: Compact top context includes model, session key, workspace/cwd, approval mode, runtime health, and active turn/task state.
- [ ] AC-3: Composer supports multiline input plus discoverable command affordances patterned after modern terminal agents: slash commands, interrupt/retry, and context/detail shortcuts.
- [ ] AC-4: Activity is shown inline with human-meaningful summaries and streaming assistant updates instead of repeated heartbeat/log lines.
- [ ] AC-5: Secondary detail surfaces for tools, artifacts, memory, cortex, routines, and sessions are collapsed by default and expandable on demand.
- [ ] AC-6: Approval and permission prompts are rendered as explicit interactive controls, not buried in transcript/event noise.
- [ ] AC-7: Resize, scroll, focus, expansion, and keyboard behaviors are deterministic and covered by tests.
- [ ] AC-8: TUI consumes the shared state protocol from Issue #3581 rather than relying primarily on raw log parsing.

## Research Anchors
- OpenCode uses a native transcript-first TUI with a compact status line, command palette style slash commands, session navigation, share/export actions, detail/thinking toggles, and keyboard-customizable actions.
- OpenCode surfaces approvals/permissions as interactive decisions instead of raw stderr text.
- OpenCode treats tools/details as contextual views, not the primary layout.
- Current Tau TUI anti-patterns:
  - shell-header and dashboard dump before the conversation starts
  - side panels consuming too much width by default
  - command prompt feel rather than persistent application structure
  - weak discoverability for interrupt/retry/session/detail actions
  - raw operational text leaking into the main experience

## Files To Touch
- `crates/tau-tui/src/main.rs`
- `crates/tau-tui/src/lib.rs`
- `crates/tau-tui/src/interactive/*`
- `crates/tau-tui/tests/tui_demo_smoke.rs`
- `scripts/run/tau-unified.sh`
- `docs/research/cli-interface-patterns-2026-03-16.md`
- `docs/guides/operator-deployment-guide.md`

## Error Semantics
- TUI must surface provider/runtime/tool failures as explicit stateful operator messages, not raw event lines.
- Approval, cancel, retry, and restart actions must always produce observable state transitions in the UI.
- If structured state is unavailable during migration, TUI must render an explicit degraded-state marker rather than silently dropping operator context.
- Layout degradation under small terminal sizes must be observable and deterministic; no hidden silent fallback to scrolling log mode.

## Test Plan
1. Add render tests for transcript-first states: idle, thinking, streaming, tool-running, approval-required, failed, cancelled, completed.
2. Add PTY/TUI interaction tests for multiline composer, slash commands, interrupt, retry, approvals, resize behavior, and detail drawer toggles.
3. Add smoke tests proving transcript remains primary while tools/memory/cortex/artifacts/sessions are still reachable.
4. Add parity tests proving TUI renders the same underlying turn/task state as webchat for core scenarios.
5. Add startup tests proving `tau-unified.sh tui` enters the transcript-first application directly rather than a shell-header/log dump mode.

## Rollout Notes
1. Finish shared state contract first.
2. Introduce transcript-first renderer behind a feature/mode gate if needed.
3. Migrate default `agent` TUI mode to the new layout once parity and PTY tests are green.
4. Remove legacy shell-header/panel-dump startup assumptions after cutover.
