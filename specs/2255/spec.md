# Spec #2255

Status: Implemented
Milestone: specs/milestones/m46/index.md
Issue: https://github.com/njfio/Tau/issues/2255

## Problem Statement

Token pre-flight checks exist in `tau-agent-core`, but runtime startup does not
derive those limits from provider/model catalog metadata. Large prompts can
reach provider APIs and fail there instead of being rejected early with a local,
actionable `TokenBudgetExceeded` error.

## Scope

In scope:

- Derive token pre-flight limits from model catalog metadata:
  - `context_window_tokens`
  - `max_output_tokens`
- Apply derived limits to local runtime and training executor agent settings.
- Add tests proving provider/model-aware pre-flight rejection behavior.

Out of scope:

- Changes to provider-side token counting algorithms.
- New CLI flags for token budgets.

## Acceptance Criteria

- AC-1: Runtime startup computes model-aware token pre-flight limits from model
  catalog metadata and passes them into `AgentConfig`.
- AC-2: When estimated input/total tokens exceed derived limits, prompt
  execution fails locally with `TokenBudgetExceeded` before provider request
  completion.
- AC-3: Training executor path uses the same derived pre-flight logic as local
  runtime path.

## Conformance Cases

- C-01 (AC-1, unit): limit-derivation helper returns expected
  `max_estimated_input_tokens` and `max_estimated_total_tokens` for representative
  model metadata.
- C-02 (AC-2, functional): a runtime agent configured with tight derived limits
  returns `TokenBudgetExceeded` for oversized prompt.
- C-03 (AC-3, integration): training executor build path sets derived limits and
  rejects oversized prompt in executor flow.

## Success Metrics / Observable Signals

- No provider call required to observe over-limit rejection for oversized
  prompts.
- Derived limits track model catalog entries instead of static defaults.
- Conformance tests pass in `tau-onboarding` and `tau-coding-agent`.
