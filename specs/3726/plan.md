# Plan: Issue #3726 - Add schema and validator for tranche-one autonomy benchmark contract

## Goal
Back the new tranche-one autonomy benchmark fixture with a durable schema and a
single local validation command.

## Approach
1. Add a JSON schema describing the benchmark fixture shape and required fields.
2. Add a validator script under `scripts/dev/` that reads the fixture and
   enforces the benchmark contract deterministically with Python stdlib only.
3. Add a focused shell contract test that exercises the schema contract, the
   happy-path validator command, and representative malformed fixtures.
4. Wire Issue `#3726` into the M334 milestone index and close the slice with
   captured verification evidence.

## Affected Modules
- `specs/3726/spec.md`
- `specs/3726/plan.md`
- `specs/3726/tasks.md`
- `specs/milestones/m334/index.md`
- `tasks/schemas/m334-tranche-one-autonomy-benchmark.schema.json`
- `scripts/dev/validate-m334-tranche-one-autonomy-benchmark.sh`
- `scripts/dev/test-m334-tranche-one-autonomy-benchmark.sh`

## Risks / Mitigations
- Risk: schema and validator drift from the actual fixture.
  Mitigation: point the validator directly at the fixture from `#3725` and keep
  the contract narrow.
- Risk: the validator implies generic JSON-schema support it does not actually
  provide.
  Mitigation: describe it explicitly as a deterministic benchmark-contract
  validator, not a general-purpose schema engine.

## Verification
- `scripts/dev/validate-m334-tranche-one-autonomy-benchmark.sh`
- `scripts/dev/validate-m334-tranche-one-autonomy-benchmark.sh --fixture /tmp/malformed.json`
- `scripts/dev/test-m334-tranche-one-autonomy-benchmark.sh`
