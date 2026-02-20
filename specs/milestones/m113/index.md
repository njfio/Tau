# M113 - Cortex Runtime Bulletin Foundation

Status: Active

## Context
M111 and M112 established Cortex gateway surfaces (`/cortex/chat`, `/cortex/status`) and observer event coverage. Remaining `G3` items in `tasks/spacebot-comparison.md` require runtime Cortex behavior: heartbeat execution, cross-session memory querying, periodic bulletin generation, and prompt-path bulletin injection.

## Source
- `tasks/spacebot-comparison.md` (G3 Cortex cross-session observer)

## Objective
Deliver a first runtime Cortex bulletin loop that can run on heartbeat cadence, build deterministic bulletin text from cross-session memory signals, and expose bulletin content to new-session prompt composition through an atomic shared pointer.

## Scope
- Introduce a Cortex runtime service in core runtime code.
- Execute Cortex updates on heartbeat cadence.
- Query memory signals across sessions for bulletin input.
- Generate and store a deterministic memory bulletin snapshot.
- Inject bulletin snapshot into new-session prompt rendering path via `ArcSwap<String>`.

## Issue Map
- Epic: #2715
- Story: #2716
- Task: #2717

## Acceptance Signals
- Heartbeat processing refreshes Cortex bulletin state with deterministic diagnostics.
- New session prompt rendering reads latest bulletin snapshot without lock contention.
- Scoped conformance/regression tests for AC/C-case mappings are green.
