# M324 - Interactive turn latency and timeout policy hardening

Status: Active

## Context
Interactive `tau-unified.sh tui` sessions can spend several minutes failing a
single trivial turn because request timeout/retry behavior is not tuned for
operator UX. The local runtime path also hardcodes a 120s agent request timeout
instead of honoring CLI timeout controls.

## Issue Hierarchy
- Epic: #3553
- Story: #3554
- Task: #3555

## Scope
- Wire local runtime agent timeout to CLI request timeout settings.
- Add explicit timeout/retry passthrough flags to `tau-tui agent`.
- Apply fast-fail interactive defaults in `tau-unified.sh tui` with override
  controls.
- Add conformance tests and docs updates for new behavior.

## Exit Criteria
- `specs/3555/spec.md` is marked `Implemented` with AC verification evidence.
- Interactive `tau-unified.sh tui` launch command includes bounded timeout/retry
  controls by default.
- Runtime honors CLI timeout for agent request timeout in local mode.
