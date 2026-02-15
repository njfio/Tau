# Issue 1740 Plan

Status: Reviewed

## Approach

1. Add an integration test in `tau-runtime` that simulates crash/restart by
   persisting a `running` background job manifest, restarting runtime, and
   verifying recovery + completion.
2. Add a standardized M24 playbook artifact template and validator script for
   resume-after-crash drills.
3. Add validator regression tests for malformed artifacts.
4. Update `docs/guides/training-ops.md` with drill procedure and artifact
   validation commands.

## Affected Areas

- `crates/tau-runtime/src/background_jobs_runtime.rs`
- `docs/guides/training-ops.md`
- `scripts/demo/m24-rl-resume-after-crash-playbook-template.json`
- `scripts/demo/validate-m24-rl-resume-after-crash-playbook.sh`
- `scripts/demo/test-m24-rl-resume-after-crash-playbook.sh`
- `specs/1740/{spec,plan,tasks}.md`

## Risks And Mitigations

- Risk: flaky restart timing in integration test.
  - Mitigation: avoid real process kill; simulate crash state via persisted
    `running` manifest and deterministic restart.
- Risk: operator artifact drift.
  - Mitigation: strict schema validator and explicit doc examples.

## ADR

No architecture/dependency/protocol change. ADR not required.
