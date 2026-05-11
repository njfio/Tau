# Spec: Issue #3757 - State-backed harness benchmark and audit panels

Status: Implemented

## Problem Statement

The `/ops/harness` UI exposes the mission harness lane, but benchmark counters
and audit history still fall back to static operator-fixture values. Operators
need the visible harness proof and review history to reflect real gateway state
when proof and audit artifacts exist.

## Scope

In scope:

- Add a dashboard harness snapshot to the Tau Ops shell context.
- Render benchmark totals, category rows, latest result text, and TUI companion
  proof summary from the harness snapshot.
- Load the latest M334 proof artifact from gateway state when present.
- Load recent harness proposal audit records from gateway state when present.
- Link Tool Execution Evidence artifact cells to the state-backed proof artifact
  when a safe harness artifact route exists.
- Render safe harness artifact views as operator proof pages with JSON metadata,
  top-level key evidence, capped payload preview, raw artifact access, and
  context-preserving return links.
- Make `view=history` an explicit audit-history summary rather than only a
  topbar acknowledgement.
- Preserve deterministic fallback values when no gateway state exists.
- Add UI and gateway integration tests proving state-backed rendering.

Out of scope:

- Provider-backed or queued benchmark scheduling.
- Live mission row discovery beyond the canonical benchmark proof.
- Autonomous self-improvement apply.

## Acceptance Criteria

### AC-1 Benchmark proof is state-backed when available

Given `/ops/harness/run-benchmark` has written the latest proof artifact,
when an operator opens `/ops/harness`,
then the benchmark panel exposes real task count, pass count, failed gate count,
category rows, proof artifact path, and latest result text from gateway state.

### AC-2 Proposal audit history is state-backed when available

Given proposal actions have written harness audit records,
when an operator opens `/ops/harness`,
then the audit table renders recent records from `ops-harness/audit.jsonl`
instead of the static fallback rows.

### AC-3 Fallback rendering remains deterministic

Given no harness state artifacts exist,
when the dashboard shell renders,
then the existing deterministic fallback M334 and proposal review markers remain
available for tests, demos, and offline rendering.

### AC-4 Tool evidence artifacts are inspectable

Given state-backed proof has tool evidence rows,
when an operator opens `/ops/harness`,
then each row's artifact cell links to the corresponding harness artifact view
instead of rendering proof evidence as inert text. When an operator opens that
artifact view, the page exposes the normalized artifact path, byte counts, JSON
kind, top-level JSON keys when present, a capped preview, a raw artifact link,
and a return link that preserves supplied harness shell/history context.

### AC-5 History view exposes audit summary proof

Given the operator opens `/ops/harness?view=history`,
when audit rows are loaded from state or fallback data,
then the route renders a dedicated history summary with row count, source,
proof-link count, selected proposal, selected action filter, latest action, and
a direct audit-log anchor. Each rendered audit row has a route-backed inspect
link, and the route renders an in-page selected audit detail panel for the
chosen audit record with a capped inline preview for safe `ops-harness/...`
proof artifacts. The history summary is promoted ahead of the default dashboard
content while the history route is active, and the legacy topbar route-action
banner is hidden so operators see one primary history heading rather than a
duplicated "Applied History" banner. State-backed history rows are scoped to
the selected proposal before the selected action filter is applied, so a
proposal-specific history route cannot mix rows from a different proposal.

## Conformance Cases

| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1, AC-3 | Functional | custom harness snapshot | shell renders | benchmark panel uses custom counts and category rows |
| C-02 | AC-2, AC-3 | Functional | custom harness snapshot | shell renders | audit table uses custom action rows |
| C-03 | AC-1, AC-2 | Integration | gateway state after benchmark/action POSTs | GET `/ops/harness` | page contains state-backed proof and audit markers |
| C-04 | AC-4 | Functional, Integration | state-backed proof artifact | shell renders and artifact route opens | tool evidence artifact cells link to `/ops/harness/artifacts/view/...`; artifact view exposes JSON metadata, proof keys, raw access, and preserved return context |
| C-05 | AC-5 | Functional, Integration | history route request | GET `/ops/harness?view=history`, `audit_action=run-benchmark`, and `audit_ref=<row-ref>` | page contains dedicated history summary promoted before the default dashboard, route-backed audit filter state scoped to the selected proposal, selected audit detail, capped proof preview, and audit anchor |

## Success Metrics / Observable Signals

- Dashboard UI tests prove custom harness snapshot values render.
- Gateway integration test proves benchmark/action state is reflected on the
  next `/ops/harness` render.
- Tool evidence rows expose state-backed proof links in both UI and gateway
  route tests.
- Artifact view route exposes JSON/top-level-key proof and preserves supplied
  harness shell/history context for return navigation.
- History view exposes state-backed audit summary counts instead of only a
  route-action banner.
- History view audit filters are route-backed and narrow state audit rows
  before the recent-row limit.
- History view scopes state-backed audit rows and total counts to the selected
  proposal before applying the action filter.
- History view selected audit details are route-backed and preserve proof
  artifact links without requiring operators to leave the harness page first.
- History view proof previews are capped, state-backed, and only read normalized
  `ops-harness/...` artifact paths.
- History route promotes the audit-history surface before the default dashboard
  instead of burying it inside the review panel.
- History route keeps a machine-readable route-action marker but hides the
  duplicate visible banner once the dedicated primary history panel is active.
- Fallback dashboard tests remain green.
- No new dependency is introduced.
