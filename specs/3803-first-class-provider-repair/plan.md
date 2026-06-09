# Plan 3803: First-Class OpenRouter Repair Adapter

## Approach

Add a focused module under the `tau_autonomous_coding_job` binary instead of
creating a new crate. The durable runtime already owns repair context creation,
provider-command execution, parsing, edit application, evidence, and verifier
retry. The adapter should therefore be a CLI-compatible provider client that
prints the same JSON contract consumed by the runtime.

## Affected Modules

- `crates/tau-coding-agent/src/bin/tau_autonomous_coding_job.rs`
  - Add `openrouter-repair-adapter`.
  - Add `--provider-repair-openrouter` policy expansion.
  - Default issue-to-merge and submit PR mode to draft.
- `crates/tau-coding-agent/src/bin/tau_autonomous_coding_job/openrouter_repair_adapter.rs`
  - Resolve env and `.env` configuration.
  - Call OpenRouter-compatible chat completions through `tau-ai`.
  - Validate strict repair output via `tau-runtime`.
  - Persist provider metadata.
- `scripts/dev/test-openrouter-repair-adapter.sh`
  - Mock OpenRouter-compatible endpoint and prove the real durable loop.
- `scripts/dev/test-autonomous-coding-gauntlet.sh`
  - Include the first-class adapter proof.

## Risks and Mitigations

- Risk: provider responses include Markdown or prose.
  - Mitigation: extract candidate JSON but still validate through
    `parse_provider_repair_edits`; invalid output fails closed.
- Risk: secrets leak into metadata.
  - Mitigation: metadata stores only auth source names and never stores key
    values.
- Risk: model naming differs between Tau and OpenRouter API ids.
  - Mitigation: accept Tau-style `openrouter/<model>` but strip the provider
    prefix before sending the API request.
- Risk: draft PR default changes operator expectations.
  - Mitigation: preserve explicit `--pr-mode pr-ready` and keep auto-merge
    gated behind `--allow-auto-merge`.

## Interfaces

- CLI:
  - `tau-autonomous-coding-job openrouter-repair-adapter`
  - `--provider-repair-openrouter`
  - `--provider-repair-env-file`
  - `--provider-repair-api-base`
  - `--provider-repair-timeout-ms`
  - `--provider-repair-max-tokens`
  - `--provider-repair-max-retries`
- Env:
  - API key: `OPENROUTER_API_KEY`, `TAU_OPENROUTER_API_KEY`,
    `OPENAI_API_KEY`, `TAU_API_KEY`
  - Model: `TAU_PROVIDER_REPAIR_MODEL`,
    `TAU_AUTONOMOUS_CODING_REPAIR_MODEL`, `TAU_PROVIDER_PROOF_MODEL`,
    `TAU_OPENROUTER_MODEL`
  - API base: `TAU_PROVIDER_REPAIR_API_BASE`,
    `TAU_AUTONOMOUS_CODING_REPAIR_API_BASE`, `TAU_OPENROUTER_API_BASE`

## Verification

- Add unit tests for env-file model resolution, OpenRouter model-id conversion,
  and strict JSON candidate validation.
- Add deterministic mock-provider shell proof through `issue-to-merge`.
- Run focused tests and clippy for the affected binary.
