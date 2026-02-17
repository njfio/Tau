# Spec #2342

Status: Accepted
Milestone: specs/milestones/m55/index.md
Issue: https://github.com/njfio/Tau/issues/2342

## Problem Statement

Roadmap claim #14 remains partial because fuzz coverage currently relies on
deterministic `cargo test` conformance cases, while AGENTS requires
`cargo-fuzz` coverage for untrusted input paths.

## Scope

In scope:

- Add `cargo-fuzz` harnesses for runtime RPC and gateway websocket parser
  surfaces.
- Add initial corpus seeds and reproducible run script(s).
- Update roadmap claim #14 with evidence once harness runs are validated.

Out of scope:

- Full fuzzing of all crates.
- Performance tuning of fuzz runtime in this slice.

## Acceptance Criteria

- AC-1: Given a local checkout, when running fuzz harness baseline command(s),
  then parser fuzz targets execute without crash/panic for configured iterations.
- AC-2: Given the harness layout, when reviewing repository structure, then
  corpus seeds and target mappings are explicit and reproducible.
- AC-3: Given roadmap updates, when reviewing claim #14 row, then status/evidence
  reflects validated cargo-fuzz coverage.

## Conformance Cases

- C-01 (AC-1, fuzz): runtime RPC raw/NDJSON and gateway WS fuzz targets run with
  non-zero iteration count and complete without crash.
- C-02 (AC-2, conformance): fuzz target/corpus directories exist with documented
  commands.
- C-03 (AC-3, conformance): `tasks/resolution-roadmap.md` claim #14 row includes
  command-level cargo-fuzz evidence.

## Dependency Approval Gate

This slice requires adding new fuzz-harness dependencies/tooling
(`cargo-fuzz` scaffolding and `libfuzzer-sys`). Per AGENTS boundary rules, this
requires explicit user approval before implementation.

Approval resolution (2026-02-17):
- Explicit approval request was made in-session.
- User directed continuation after the approval request; this is treated as
  approval to proceed with dependency/tooling additions for this issue.
