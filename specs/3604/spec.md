# Spec: Issue #3604 - Surface mutating tool evidence status in the interactive TUI

Status: Planned

## Objective
Show explicit mutating-tool-evidence status in the main interactive TUI chat surface during active build/create turns so operators can tell whether Tau is still read-only, has not used tools yet, or has already produced successful mutating evidence.

## Inputs/Outputs
Inputs:
- Current interactive app state: latest user prompt, agent state, and tool entries.
- Tool entry statuses and names.

Outputs:
- A visible status banner in the main chat panel during active build/create turns.
- Banner text that distinguishes no tool evidence, read-only evidence, and mutating evidence.

## Boundaries / Non-goals
In scope:
- Current `master` interactive ratatui shell only.
- Visibility in the main chat surface.
- Integration-style render tests using the real interactive render path.

Out of scope:
- Runtime safety policy changes.
- Tool execution semantics.
- Full TUI redesign.
- File previews, diffs, or artifact panes.

## Failure Modes
- Build/create turn is active and there are no successful tool results yet.
- Build/create turn is active and only successful non-mutating tool results exist.
- Build/create turn is active and successful mutating tool results exist.
- Non-build turns or idle turns incorrectly show the banner.

## Acceptance Criteria
### AC-1 Active build turns show missing-mutation state
Given an active build/create turn and no successful tool evidence for the current turn,
when the chat panel renders,
then the main chat surface includes `no mutating evidence yet`.

### AC-2 Active build turns show read-only state
Given an active build/create turn and only successful non-mutating tool evidence for the current turn,
when the chat panel renders,
then the main chat surface includes `read-only so far`.

### AC-3 Active build turns show confirmed mutation state
Given an active build/create turn and successful `write` or `edit` evidence for the current turn,
when the chat panel renders,
then the main chat surface includes `mutating evidence confirmed`.

### AC-4 Non-build or idle turns omit the banner
Given a non-build turn or an idle app state,
when the chat panel renders,
then the mutating-evidence banner is absent.

### AC-5 Real render path coverage exists for each state
Given the interactive ratatui renderer,
when tests render the real frame,
then there is at least one render-path test for AC-1 through AC-4.

## Files To Touch
- `specs/3604/spec.md`
- `crates/tau-tui/src/interactive/app.rs`
- `crates/tau-tui/src/interactive/mod.rs`
- `crates/tau-tui/src/interactive/ui.rs`
- `crates/tau-tui/src/interactive/build_status.rs`

## Error Semantics
- No silent fallback banner text.
- If build-status classification cannot prove a build/create turn is active, the banner is omitted.
- Classification is pure and non-throwing; entrypoints continue to handle operational errors.

## Test Plan
- Red render tests for AC-1 through AC-4 using `ratatui::backend::TestBackend`.
- Unit tests for prompt classification and tool-evidence classification.
- `cargo test -p tau-tui` after integration.
