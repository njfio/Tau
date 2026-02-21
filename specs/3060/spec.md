# Spec: Issue #3060 - ops memory delete confirmation contracts

Status: Implemented

## Problem Statement
The Tau Ops memory route currently supports search/create/edit contracts but does
not expose deterministic delete-confirmation contracts. PRD checklist item
`2082` ("Delete memory entry with confirmation") remains unverified.

## Scope
In scope:
- Add deterministic delete-entry form markers on `/ops/memory`.
- Enforce explicit confirmation semantics for delete submissions.
- Persist delete behavior through existing gateway memory runtime.
- Surface deterministic delete status/result markers after redirect.

Out of scope:
- Memory graph rendering changes.
- New memory relation/detail panels.
- New external dependencies.

## Acceptance Criteria
### AC-1 Memory route exposes deterministic delete confirmation form contracts
Given `/ops/memory` renders,
when an operator opens the route,
then deterministic delete form controls and confirmation markers exist.

### AC-2 Delete submission requires explicit confirmation and target entry
Given a delete form submission,
when confirmation is missing or target entry is absent,
then no delete occurs and route status remains deterministic idle/error.

### AC-3 Successful delete removes entry and emits deterministic status markers
Given an existing memory entry and confirmed delete submission,
when the form is submitted,
then the entry is removed and redirect/status markers identify delete success.

### AC-4 Existing memory contracts remain green
Given search/scope/type/create/edit contracts,
when delete contracts are added,
then prior behavior and conformance suites remain green.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
|---|---|---|---|---|---|
| C-01 | AC-1 | Functional | `/ops/memory` route render | inspect SSR output | delete form + confirmation markers are present |
| C-02 | AC-2 | Integration | missing confirm or missing entry | submit delete form | redirect shows idle/error status and entry remains unchanged |
| C-03 | AC-3 | Integration | existing entry + confirm delete | submit delete form | entry is deleted and status markers show deleted target |
| C-04 | AC-4 | Regression | existing memory suites | rerun selected specs | prior contracts remain green |

## Success Metrics / Signals
- `cargo test -p tau-dashboard-ui spec_3060 -- --test-threads=1`
- `cargo test -p tau-gateway spec_3060 -- --test-threads=1`
- `cargo test -p tau-gateway spec_2905 -- --test-threads=1`
- `cargo test -p tau-gateway spec_2909 -- --test-threads=1`
- `cargo test -p tau-gateway spec_2913 -- --test-threads=1`
- `cargo test -p tau-gateway spec_2917 -- --test-threads=1`
- `cargo test -p tau-gateway spec_2921 -- --test-threads=1`
- `cargo fmt --check`
- `cargo clippy -p tau-dashboard-ui -p tau-gateway -- -D warnings`
