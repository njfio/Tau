# Test Coverage Targets

## Purpose

This guide makes the quality target for under-tested crates explicit. It does
not replace the issue-level spec contract; every implementation issue still maps
acceptance criteria to conformance cases and tests under `specs/<issue-id>/`.

## Target Thresholds

| Crate or path | Risk tier | Target before release | Required evidence |
|---|---|---:|---|
| `tau-gateway` | P0 operator/control plane | >= 90% line coverage on changed modules | AC -> C-case -> `cargo test -p tau-gateway <spec-id>` plus regression tests for auth/error/state paths |
| `tau-dashboard-ui` | P0 operator surface | >= 85% line coverage on changed render modules | SSR marker tests for each changed route and one route-level gateway regression when served through `tau-gateway` |
| `tau-provider` | P0 credential/provider routing | >= 85% line coverage on changed modules | credential success/failure/fallback tests; no secret values in snapshots or logs |
| `tau-runtime` and `tau-orchestrator` | P0 process/concurrency paths | >= 85% line coverage on changed modules | lifecycle, cancellation, timeout, and restart/recovery tests |
| `tau-memory` and `tau-session` | P1 durable state | >= 80% line coverage on changed modules | migration/load/save tests plus corruption and empty-state regressions |
| Training crates | P1 long-running jobs | >= 75% line coverage on changed modules | job-state, retry, and report-shape tests |
| Deprecated compatibility crates | P2 compatibility only | no coverage decrease | compile/regression coverage for public compatibility behavior |

Where line coverage tooling is unavailable in CI, use scoped conformance counts
as the release gate: every AC must map to at least one named C-case and at least
one test command in the issue spec.

## Conformance Mapping

Each spec should contain:

- `AC-n` acceptance criteria written as observable behavior.
- `C-nn` conformance cases mapped to ACs.
- Test names that include the spec id or case id where practical.
- A tier matrix with `N/A` explanations for omitted property, fuzz, mutation,
  or performance gates.

## Property and Concurrency Growth Targets

Add property or concurrency coverage when a change touches:

- ranking order, score normalization, or tie-breaking;
- compaction or summarization boundaries;
- process lifecycle, cancellation, or restart supervision;
- credential resolution fallbacks;
- durable state loading after partial writes or malformed records.

Minimum expectation for those paths is one deterministic regression test plus
one randomized/property or concurrent interleaving test when the behavior has a
stable invariant.

## Release Review

Before release, reviewers should sample changed specs and verify:

1. No changed P0/P1 module has untested ACs.
2. Coverage exceptions have a follow-up issue and an explicit risk owner.
3. `cargo test -p <crate> <spec-id>` commands named in specs still run.
4. Generated artifacts and docs reference current crate names and routes.
