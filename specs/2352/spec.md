# Spec #2352

Status: Accepted
Milestone: specs/milestones/m57/index.md
Issue: https://github.com/njfio/Tau/issues/2352

## Problem Statement

`README.md` contains stale provider capability wording that no longer reflects
current merged behavior (notably omission of first-class `openrouter/*` support
and current alias routing language).

## Scope

In scope:

- Update stale provider/runtime capability statements in `README.md`.
- Keep existing correct sections intact.
- Validate local README links/paths after edits.

Out of scope:

- Broad content rewrite of docs.
- Behavior changes in crates/scripts.

## Acceptance Criteria

- AC-1: Given the provider/runtime capability summary in README, when comparing
  it to current `tau-ai` provider parsing and model-catalog behavior, then
  README explicitly reflects first-class `openrouter/*` plus alias support.
- AC-2: Given README quickstart/capability docs links, when validating
  repository-relative paths, then all local links resolve to existing files.
- AC-3: Given this docs slice, when running formatting checks, then workspace
  formatting remains clean.

## Conformance Cases

- C-01 (AC-1, conformance): README provider routing bullet includes
  `openrouter/*` and alias note aligned with `crates/tau-ai/src/provider.rs`.
- C-02 (AC-2, conformance): local Markdown links in README resolve with no
  missing repository-relative paths.
- C-03 (AC-3, conformance): `cargo fmt --check` passes.
