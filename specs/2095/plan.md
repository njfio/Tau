# Plan #2095

Status: Implemented
Spec: specs/2095/spec.md

## Approach

1. Use merged implementation evidence from `#2096/#2097` as source of truth.
2. Create story-level lifecycle artifacts with AC/conformance mapping.
3. Re-run scoped verification commands to confirm no drift on latest `master`.
4. Close story and hand off to epic roll-up.

## Affected Modules

- `specs/2095/spec.md`
- `specs/2095/plan.md`
- `specs/2095/tasks.md`
- `specs/2096/spec.md`
- `specs/2097/spec.md`

## Risks and Mitigations

- Risk: story closure loses traceability to task-level evidence.
  - Mitigation: explicitly map ACs to `#2096/#2097` verification commands.
- Risk: drift after merge before roll-up.
  - Mitigation: rerun task-scoped verification on latest `master`.

## Interfaces and Contracts

- `bash scripts/dev/test-cli-args-domain-split.sh`
- `cargo check -p tau-cli --lib --target-dir target-fast`
- `cargo test -p tau-coding-agent startup_preflight_and_policy --target-dir target-fast`

## ADR References

- Not required.
