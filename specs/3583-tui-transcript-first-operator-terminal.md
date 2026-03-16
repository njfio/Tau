# Spec: Issue #3583 - Redesign Tau TUI as a Transcript-First Operator Terminal

Issue: https://github.com/njfio/Tau/issues/3583
Status: Planned

## Objective
Redesign Tau TUI from a panel-heavy operator/debug shell into a transcript-first terminal agent interface. The result must support fast, readable operator workflows with compact session context, slash commands, approvals, interrupts, retries, meaningful progress, and direct access to tools, tasks, artifacts, memory, cortex, and routines without burying the conversation.

## Inputs/Outputs
- Inputs:
  - Shared operator turn/task state protocol from Issue #3581.
  - Local TUI command input and keyboard shortcuts.
  - Runtime approval, tool, artifact, memory, cortex, and job state updates.
- Outputs:
  - A transcript-first TUI layout with operator-centric task/tool context.
  - A compact status/header model surfacing model, auth, cwd/session, provider/runtime health, and active task state.
  - Deterministic shortcut, focus, scroll, expand/collapse, approval, interrupt, retry, and artifact navigation behavior.

## Boundaries / Non-goals
- No webchat redesign in this issue.
- No independent UI-only event model; TUI must consume the shared state contract.
- No swarm orchestration implementation beyond terminal hooks and navigation affordances.
- No fallback to panel-dump/dashboard behavior as the primary interaction mode.

## Failure Modes
1. Long-running turn with tools and partial assistant output.
   - Expected: transcript remains primary, progress is visible in-place, tool/task state is accessible without hiding the conversation.
2. Runtime/provider failure during an active turn.
   - Expected: failure is rendered inline with actionable recovery controls and no TUI exit.
3. Small terminal size or resize during active work.
   - Expected: layout degrades gracefully, preserves input usability, and keeps transcript legible.
4. Approval-required turn.
   - Expected: approval state is visually distinct, keyboard-accessible, and does not get lost in generic event logs.
5. Operator wants memory/cortex/tool/task detail.
   - Expected: detail panes/overlays are available without displacing the core transcript workflow.

## Acceptance Criteria (testable booleans)
- [ ] AC-1: TUI main interaction is transcript-first rather than panel-first.
- [ ] AC-2: Compact top status context includes model, auth/approval mode, cwd/session, health, and active task state.
- [ ] AC-3: Composer supports multiline input, slash commands, visible shortcuts, interrupt, retry, and approval actions.
- [ ] AC-4: Progress/failure states render in-place with human-meaningful summaries rather than repeated heartbeat/log spam.
- [ ] AC-5: Tools, artifacts, memory, cortex, and routines are available via focused secondary surfaces without obscuring the main transcript by default.
- [ ] AC-6: Resize, scroll, focus, expansion, and keyboard behaviors are deterministic and covered by tests.
- [ ] AC-7: TUI consumes the shared state protocol from Issue #3581 rather than relying primarily on raw log parsing.

## Files To Touch
- `crates/tau-tui/src/main.rs`
- `crates/tau-tui/src/lib.rs`
- `crates/tau-tui/tests/tui_demo_smoke.rs`
- `crates/tau-coding-agent/src/runtime_loop.rs`
- `scripts/run/tau-unified.sh`
- `docs/guides/operator-deployment-guide.md`
- `docs/research/cli-interface-patterns-2026-02-26.md`

## Error Semantics
- TUI must surface provider/runtime/tool failures as explicit stateful operator messages, not just raw event lines.
- Approval, cancel, retry, and restart actions must always produce observable state transitions in the UI.
- If structured state is unavailable during migration, TUI must render an explicit degraded-state marker rather than silently dropping operator context.

## Test Plan
1. Add parser/render tests for transcript-first layout states: idle, thinking, tool-running, approval-required, partial-output, failed, cancelled, completed.
2. Add PTY/TUI interaction tests for multiline composer, slash commands, interrupt, retry, approvals, and resize behavior.
3. Add smoke tests proving transcript remains primary while tools/memory/cortex/artifacts are still reachable.
4. Add parity tests proving TUI renders the same underlying turn/task state as webchat for core scenarios.

## Rollout Notes
1. Finish shared state contract first.
2. Introduce transcript-first renderer behind a feature/mode gate if needed.
3. Migrate default `agent` TUI mode to the new layout once parity and PTY tests are green.
4. Remove legacy panel-dump startup/output assumptions after cutover.
