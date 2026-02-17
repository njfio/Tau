# Spec #2261

Status: Implemented
Milestone: specs/milestones/m46/index.md
Issue: https://github.com/njfio/Tau/issues/2261

## Problem Statement

Gap-8 requires an explicit dashboard product direction (standalone dashboard vs.
gateway consolidation) and validated implementation evidence. The repository now
contains both `tau-dashboard` runtime processing and gateway dashboard API/SSE
surfaces, but the decision contract and closure artifact set are missing.

## Scope

In scope:

- Define and document dashboard direction for M46.
- Validate dashboard functionality on the selected path with tests.
- Add issue lifecycle artifacts for `#2261`.

Out of scope:

- New dashboard UI redesign.
- Deleting crates or changing package topology.

## Acceptance Criteria

- AC-1: Given dashboard direction decision, when docs are reviewed, then they
  explicitly state gateway-backed consolidation as the production access path.
- AC-2: Given gateway dashboard endpoints, when integration tests run, then
  health/widgets/timeline/alerts/actions/stream behavior remains functional.
- AC-3: Given dashboard runtime contract logic, when `tau-dashboard` tests run,
  then deterministic runner/contract behavior remains functional.
- AC-4: Given issue closure requirements, when `#2261` is closed, then
  `specs/2261/{spec,plan,tasks}.md` is finalized and issue status is done.

## Conformance Cases

- C-01 (AC-1, documentation): `docs/guides/dashboard-ops.md` includes explicit
  direction statement that gateway API/SSE is the production dashboard path.
- C-02 (AC-2, integration): `cargo test -p tau-gateway` dashboard endpoint tests
  pass.
- C-03 (AC-3, integration): `cargo test -p tau-dashboard` passes.
- C-04 (AC-4, governance): issue `#2261` closes with finalized lifecycle
  artifacts and `status:done`.

## Success Metrics / Observable Signals

- Dashboard direction statement exists in runbook.
- Gateway and dashboard crates pass dashboard-related test coverage.
- Issue `#2261` transitions to done with merged PR evidence.
