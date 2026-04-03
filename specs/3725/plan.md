# Plan: Issue #3725 - Define tranche-one autonomy benchmark contract and task set

## Goal
Create the first durable benchmark contract for tranche-one Tau autonomy so the
rest of the vertical slice can target a stable, reviewable success bar.

## Approach
1. Encode the tranche-one benchmark as a machine-readable fixture under
   `tasks/fixtures/m334/`.
2. Include four representative tasks:
   - governed repo/spec-to-PR build
   - greenfield build
   - research/design
   - data-to-deliverable
3. Encode suite-level policy for allowed checkpoints, disallowed human steering,
   and deferred non-goals.
4. Add the task to the M334 milestone index so the benchmark contract is part of
   the Ralph-loop architecture container.

## Affected Modules
- `tasks/fixtures/m334/tranche-one-autonomy-benchmark.json`
- `specs/milestones/m334/index.md`
- `specs/3725/spec.md`
- `specs/3725/plan.md`
- `specs/3725/tasks.md`

## Risks / Mitigations
- Risk: the suite is too vague to guide later autonomy implementation.
  Mitigation: require concrete goals, deliverables, checkpoint classes, and
  pass requirements per task.
- Risk: the suite optimizes for abstract demos instead of Tau's strongest near
  term usefulness domain.
  Mitigation: include one governed repo/spec-to-PR task alongside the user's
  broader examples.
- Risk: later work quietly expands tranche-one scope.
  Mitigation: encode deferred non-goals directly in the benchmark fixture.

## Verification
- `python3 - <<'PY'`
  `import json`
  `from pathlib import Path`
  `path = Path('tasks/fixtures/m334/tranche-one-autonomy-benchmark.json')`
  `data = json.loads(path.read_text())`
  `assert len(data['tasks']) == 4`
  `assert {task['category'] for task in data['tasks']} == {'repo_build', 'greenfield_build', 'research_design', 'data_to_deliverable'}`
  `assert data['suite_policy']['allowed_operator_interventions'] == ['provider_auth', 'major_direction_choice']`
  `for task in data['tasks']:`
  `    assert task['goal']`
  `    assert task['required_deliverables']`
  `    assert task['allowed_checkpoints']`
  `    assert task['pass_requirements']`
  `print('benchmark fixture validated')`
  `PY`
- `rg -n "#3725|benchmark contract and task set" specs/milestones/m334/index.md`
