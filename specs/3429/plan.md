# Plan: Issue #3429 - Integrated reliability and recovery proof

## Approach
1. Add RED test harness for a new integrated reliability verification script (`scripts/verify/test-m296-integrated-reliability-wave.sh`).
2. Implement `scripts/verify/m296-integrated-reliability-wave.sh` with explicit step list and report generation, fail-closed behavior, and per-step logs.
3. Re-run harness and targeted regression checks to confirm pass/fail behavior and report contract.
4. Capture command evidence in `specs/3429/tasks.md`.

## Implementation Targets
- `scripts/verify/m296-integrated-reliability-wave.sh`
- `scripts/verify/test-m296-integrated-reliability-wave.sh`
- `specs/milestones/m296/index.md`
- `specs/3429/spec.md`
- `specs/3429/plan.md`
- `specs/3429/tasks.md`

## Risks and Mitigations
- Risk: verification suite becomes too slow/noisy for iteration.
  - Mitigation: keep suite command list explicit and bounded; capture per-step logs for quick diagnosis.
- Risk: report structure drifts and breaks downstream automation.
  - Mitigation: add deterministic harness assertions on schema/fields/step ids.
- Risk: partial failures mask root cause.
  - Mitigation: fail-closed overall status, preserve each step status/log path in report.

## Interfaces / Contracts
- Verification script CLI contract and report JSON shape.
- Step ids in report payload (`restart`, `reconnect`, `degraded dependency`, `auth fail-closed`, `rl`).
- Fail-closed exit semantics.

## ADR
Not required (no architecture/dependency/protocol decision change).
