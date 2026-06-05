# Spec: Issue #3764 - Agent Canvas proof loop stores comparison evidence

Status: Implemented
Priority: P1
Milestone: M334 - Tau Ralph loop supervisor architecture
Parent: #3654

## Problem Statement

Agent Canvas v2 can render generated HTML artifacts and expose diagnostics
markers, but the repeatable proof stops at validating the surface. To make the
canvas a flagship agent loop, the proof must show a closed iteration: generate an
artifact, inspect route/runtime evidence, apply one targeted deterministic fix,
rerun the proof, and persist before/after comparison evidence.

## Scope

In scope:

- Extend `scripts/dev/ops-chat-canvas-proof.sh` with deterministic proof-loop
  evidence.
- Preserve the existing `/ops/chat/send` recovery, artifact generation, and
  Agent Canvas v2 runtime contract checks.
- Persist before/after artifact hashes, route contract markers, diagnostics
  capability checks, targeted fix metadata, and rerun comparison status in JSON.
- Add shell regression coverage for the proof-loop JSON contract.
- Record this issue's artifact path explicitly because `specs/3764/` already
  contains an older implemented local spec.

Out of scope:

- Adding browser dependencies to CI.
- Changing the Agent Canvas product UI layout.
- Reworking the gateway agent runtime or provider/tool selection.
- Claiming true pixel values from a headed browser when the proof is running in
  shell-only CI mode.

## Acceptance Criteria

AC-1: Given the ops-chat canvas proof script completes, when the proof JSON is
written, then it contains a `proof_loop.result = "passed"` section with before
and after iterations.

AC-2: Given the generated HTML artifact exists, when the proof loop runs, then it
records the before artifact hash, applies one deterministic targeted fix, records
the after artifact hash, and reports that the artifact changed.

AC-3: Given the `/ops/chat` route is fetched before and after the targeted fix,
when the route HTML is inspected, then both iterations preserve Agent Canvas DOM,
console, pixel, screenshot, controlled interaction, and artifact-history
contract markers.

AC-4: Given the proof loop reruns after the fix, when comparison evidence is
written, then it reports the rerun contract as stable and the targeted fix marker
as visible in the artifact.

## Conformance Cases

| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Functional | fake live gateway shell test | run proof script | JSON includes `proof_loop.result = "passed"` and two iterations |
| C-02 | AC-2 | Functional | generated artifact file | proof loop applies fix | before/after SHA-256 values differ and `artifact_changed = true` |
| C-03 | AC-3 | Conformance | fake `/ops/chat` route response | route fetched before and after | both iterations record Agent Canvas diagnostics contracts as true |
| C-04 | AC-4 | Regression | fixed artifact | rerun proof evidence emitted | comparison reports stable route contract and visible targeted fix |

## Success Metrics / Observable Signals

- `scripts/dev/test-ops-chat-canvas-proof.sh`
- `scripts/dev/ops-chat-canvas-proof.sh --base-url <local gateway> --session <key> --artifact-path <workspace html> --output-json tasks/reports/ops-chat-canvas-proof-loop.json`
- `git diff --check`
- Proof JSON contains machine-readable before/after hashes and comparison status.
