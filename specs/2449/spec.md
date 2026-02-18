# Spec #2449 - implement G7 lifecycle metadata and soft-delete flow

Status: Accepted
Milestone: specs/milestones/m76/index.md
Issue: https://github.com/njfio/Tau/issues/2449

## Problem Statement

Current memory records lack lifecycle access tracking and soft-delete
semantics, so operators cannot retire stale records safely while preserving
history.

## Scope

In scope:

- Add lifecycle metadata fields to runtime memory records.
- Update read/search/list behavior to ignore forgotten records by default.
- Add a soft-delete operation in runtime + tool layers.

Out of scope:

- Automated decay/pruning jobs and duplicate/orphan maintenance.

## Acceptance Criteria

- AC-1: Runtime record schema includes lifecycle metadata with backward-safe
  defaults for legacy records.
- AC-2: Soft-deleted records are excluded from default read/search/list flows.
- AC-3: Memory tooling exposes a deterministic delete operation and reason
  codes.
- AC-4: Conformance and regression tests cover active/forgotten lifecycle paths.

## Conformance Cases

- C-01 (AC-1, unit): legacy record JSON deserializes with lifecycle defaults.
- C-02 (AC-2, integration): forgotten records are absent from read/search/list.
- C-03 (AC-3, functional): `memory_delete` marks record forgotten and returns
  success payload.
- C-04 (AC-4, conformance): RED/GREEN tests for lifecycle metadata touch and
  forgotten filtering pass.
