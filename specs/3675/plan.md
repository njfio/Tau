# Plan: Force Concrete Mutating Tool Choice On Ralph-Loop Recovery Retries

## Approach

Add a small gateway helper that selects the retry `ToolChoice` for mutation-recovery turns from:

1. original task intent tokens
2. registered tool names
3. a conservative mapping

Initial mapping:
- create/build/make/scaffold/generate/new-folder style prompts -> `write`
- otherwise -> generic `Required`

The helper will only run on retry turns where mutation evidence is still missing. It will not affect first attempts or validation-only retries.

## Affected Modules

- `crates/tau-gateway/src/gateway_openresponses/openresponses_execution_handler.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`

## Risks

- Over-constraining retries to `write` when `edit` would be more natural.
  - Mitigation: limit the heuristic to create/build/new-file style tasks and retain fallback to `Required`.
- Divergence between retry prompt text and tool choice.
  - Mitigation: keep the existing retry prompt text, only tighten the tool choice field.

## Verification

- unit/conformance coverage for the retry tool-choice selector
- update the existing read-only saturation and timeout-after-read-only retry regressions
- rerun the scoped gateway retry/timeout regressions
