# Spec #2068

Status: Accepted
Milestone: specs/milestones/m25/index.md
Issue: https://github.com/njfio/Tau/issues/2068

## Problem Statement

M25 velocity work needs a reproducible build/test timing baseline that records
durations for key local/CI commands under explicit environment metadata.
Without a deterministic baseline artifact (JSON + Markdown), hotspot
attribution and later latency-budget enforcement (`#2048`) cannot be verified
or compared across runs.

## Acceptance Criteria

- AC-1: A deterministic timing-matrix artifact generator produces JSON +
  Markdown reports containing per-command duration statistics and ranking.
- AC-2: The artifact includes environment metadata (OS/arch/shell/toolchain)
  and source mode (`fixture` or `live`) for reproducibility.
- AC-3: Baseline artifacts rank top latency hotspots (highest average duration)
  and include reproducible command invocations for each measured row.

## Scope

In:

- Add a build/test latency baseline generator script with fixture mode and live
  command execution mode.
- Emit machine-readable + Markdown reports under `tasks/reports/`.
- Add schema/contract/functional tests covering shape, determinism, and hotspot
  ranking.
- Add a guide describing invocation and artifact interpretation.

Out:

- CI cache-key tuning and parallel scheduling implementation (`#2070`/`#2047`).
- Latency budget policy enforcement (`#2071`/`#2048`).

## Conformance Cases

- C-01 (AC-1, functional): fixture-mode run emits JSON + Markdown containing at
  least one command row with `count/avg_ms/p50_ms/min_ms/max_ms`.
- C-02 (AC-2, integration): JSON artifact includes environment metadata and
  `source_mode`, and schema validation succeeds.
- C-03 (AC-3, regression): hotspot table is sorted by descending `avg_ms`; ties
  are stable by command id.
- C-04 (AC-1/AC-2, regression): malformed fixture or missing command specs fail
  closed with a non-zero exit and actionable error.

## Success Metrics

- `tasks/reports/m25-build-test-latency-baseline.{json,md}` generated
  deterministically from fixture mode.
- Contract/functional/regression tests pass for both valid and fail-closed
  inputs.
- `#2068` can be closed with artifact evidence that `#2045` can consume.
