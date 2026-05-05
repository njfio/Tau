# Plan 3778: Keep Proposal Safety Summary Visible

## Approach

Add a conformance test for compact first-viewport proposal detail. Reorder the
proposal detail rows so the approval-relevant safety summary comes first, then
constrain the detail section with a compact internal scroll area. This keeps the
policy-to-proposal decision path visible without changing endpoints, proposal
content, audit history, or apply semantics.

## Affected Modules

- `crates/tau-dashboard-ui/src/lib.rs`
- `crates/tau-dashboard-ui/src/tests.rs`

## Risks and Mitigations

- Risk: Compacting proposal detail could hide useful context.
  Mitigation: Keep every existing row and make the detail section internally
  scrollable.
- Risk: Reordering rows could obscure root-cause context.
  Mitigation: Move only the operator decision summary ahead of lower-priority
  explanation; preserve the rest of the content.

## Interfaces

No route, endpoint, schema, approval, audit, or self-improvement semantics
change. This is a dashboard layout and marker contract only.
