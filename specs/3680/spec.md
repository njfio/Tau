# Spec: Issue #3680 - Add local session persistence to `tau-tui` REPL

Status: Not Integrated (2026-04-23 reclassification — see below)

> **2026-04-23 status correction**: although this spec was previously marked
> "Implemented", an audit discovered that the implementation modules
> (`session_state.rs` + `session_state_tests.rs`) were introduced to the tree
> in commit `8926bd4a` but were **never referenced from any `mod` declaration
> in any git revision**. The compiler never saw them; the feature shipped zero
> runtime behavior. The orphan files were removed in the audit-follow-up
> cleanup. Re-implementing this feature is pending a dedicated spec cycle;
> see `docs/solutions/patterns/fallibility-audit-workspace-2026-04.md`
> (Category A) for the forensic trail.

## Problem Statement
`tau-tui` now has runtime control, prompt history, and transcript search/copy,
but it still loses too much operator state on restart. Draft input, prompt
history, and active mission/session binding vanish between launches, which
makes the REPL feel disposable instead of durable. Tau needs a persistence
slice that restores core local REPL state from disk without any gateway
changes.

## Scope
In scope:
- local persistence for core interactive TUI state under `.tau/tui/`
- restore draft input, prompt history, and active mission/session binding on
  startup
- save state during interaction and on exit without changing gateway protocols
- TDD coverage for restore/save behavior
- spec/plan/tasks updates under `specs/3680/`
- milestone update in `specs/milestones/m335/index.md`

Out of scope:
- server-side or gateway persistence changes
- full transcript persistence
- cross-machine sync or multi-profile merge
- restoring ephemeral render-only state like focus or tool panel width

## Acceptance Criteria
### AC-1 Core local REPL state is restored on startup
Given a previous `tau-tui interactive` session wrote local state to disk,
when the TUI starts again,
then draft input, prompt history, active mission id, and session key are
restored into the new app instance.

### AC-2 Local state is saved without changing gateway behavior
Given the operator edits input, submits prompts, or changes active mission
binding,
when the TUI persists local state,
then it writes the updated snapshot to a local state file and existing gateway
request behavior remains unchanged.

### AC-3 Missing or invalid local state fails soft
Given the local state file is missing or contains invalid JSON,
when the TUI starts,
then Tau falls back to a clean interactive session instead of crashing.

### AC-4 Earlier M335 REPL slices do not regress
Given runtime control, prompt history, and transcript power already exist,
when local persistence is added,
then those behaviors still pass in scoped regression coverage.

## Conformance Cases
- C-01 A saved local session snapshot restores draft input, prompt history, and
  active mission/session binding into a new app. Maps to AC-1. Tier:
  Functional.
- C-02 After local state mutations, Tau writes a JSON snapshot under
  `.tau/tui/interactive-session.json`. Maps to AC-2. Tier: Functional.
- C-03 Invalid or missing session-state files are ignored safely and the app
  starts clean. Maps to AC-3. Tier: Functional.
- C-04 Existing `#3677`, `#3678`, and `#3679` TUI regression tests continue to
  pass after persistence lands. Maps to AC-4. Tier: Regression.

## Success Metrics / Observable Signals
- Draft input survives a restart
- Prompt history survives a restart
- Resumed mission/session context survives a restart
- Broken session-state files do not crash the TUI

## Key Decisions
- Persistence is local-only and file-backed under `.tau/tui/`
- The snapshot should include only operator leverage state, not the full
  transcript or tool stream
- Invalid state is treated as recoverable and ignored
