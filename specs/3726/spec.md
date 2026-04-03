# Spec: Issue #3726 - Add schema and validator for tranche-one autonomy benchmark contract

Status: Implemented

## Problem Statement
Issue `#3725` created the tranche-one autonomy benchmark fixture, but the
fixture is still only convention-validated. Tau needs a durable schema and a
deterministic validator so later mission-mode and autopilot work cannot silently
degrade the benchmark contract.

## Scope
In scope:
- a JSON schema for `tasks/fixtures/m334/tranche-one-autonomy-benchmark.json`
- a deterministic validator script under `scripts/dev/`
- focused shell contract coverage for the schema and validator assets
- milestone wiring updates for the M334 issue hierarchy
- spec/plan/tasks updates under `specs/3726/`

Out of scope:
- runtime benchmark execution or mission-result classification

## Acceptance Criteria
### AC-1 A durable schema exists for the tranche-one autonomy benchmark fixture
Given the benchmark contract from `#3725`,
when maintainers inspect the benchmark schema,
then it declares the required top-level fields, suite policy fields, success bar
fields, and task fields for the tranche-one autonomy fixture.

### AC-2 A deterministic validator rejects malformed benchmark fixtures
Given the validator script,
when maintainers run it against the benchmark fixture,
then it exits successfully for the current fixture and fails closed for missing
required fields, unsupported categories, or invalid operator checkpoint rules.

### AC-3 The validator keeps the tranche-one intervention model explicit
Given the validator script,
when it checks the benchmark fixture,
then it enforces the tranche-one checkpoint model:
- allowed operator interventions are only `provider_auth` and
  `major_direction_choice`
- task checkpoints must remain within the allowed set
- routine human steering cannot be part of a passing contract

## Conformance Cases
- C-01 The schema declares the required benchmark top-level fields and task
  field groups. Maps to AC-1. Tier: Conformance.
- C-02 Running the validator against the current fixture succeeds. Maps to AC-2.
  Tier: Functional.
- C-03 The validator rejects a mutated fixture with an unsupported task category
  or checkpoint rule. Maps to AC-2 and AC-3. Tier: Regression.

## Success Metrics / Observable Signals
- The tranche-one benchmark contract is now backed by a durable schema file
- Maintainers can validate the benchmark fixture with one deterministic command
- The allowed human-checkpoint model is enforced by tooling rather than prose

## Key Decisions
- Keep the first validator deterministic and local; do not require network
  access or third-party Python packages.
- Validate the current benchmark contract directly instead of attempting a
  generic schema engine for all repo artifacts.
