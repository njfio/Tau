# Spec #2219

Status: Implemented
Milestone: specs/milestones/m42/index.md
Issue: https://github.com/njfio/Tau/issues/2219

## Problem Statement

The roadmap in `tasks/resolution-roadmap.md` identifies immediate P0 gaps in
provider/model readiness: stale built-in model catalog entries, missing
DeepSeek alias/key wiring, and no local-safe recipe for live provider
validation.

## Acceptance Criteria

- AC-1: `ModelCatalogEntry` supports additional metadata needed by current models:
  `supports_extended_thinking`, `max_output_tokens`, `knowledge_cutoff`,
  `deprecated`, and `cached_input_cost_per_million` (serde-compatible defaults preserved).
- AC-2: Built-in model catalog is refreshed to remove duplicate/deprecated legacy entries and include new frontier/legacy entries from the roadmap quick-win set.
- AC-3: DeepSeek alias parsing and API-key candidate resolution are supported (`deepseek` provider alias + `DEEPSEEK_API_KEY` and `TAU_DEEPSEEK_API_KEY`).
- AC-4: Local-safe key injection and live validation workflow is documented and executable from repo scripts without committing secrets.
- AC-5: Spec-derived RED/GREEN test evidence exists and targeted crate tests pass.

## Scope

In:

- `crates/tau-provider/src/model_catalog.rs`
- `crates/tau-ai/src/provider.rs`
- `crates/tau-provider/src/auth.rs`
- `crates/tau-cli/src/cli_args.rs`
- local-safe provider key template/run script under `scripts/dev/`
- quickstart/provider docs updates for local key placement

Out:

- first-class `Provider::OpenRouter` enum variant and dedicated client type
- remote catalog auto-refresh endpoint design
- PPO training algorithm integration

## Conformance Cases

- C-01 (AC-1, unit/conformance): serialization/deserialization roundtrip for `ModelCatalogEntry` includes new fields with backward-compatible defaults.
- C-02 (AC-2, conformance): built-in catalog contains required new models (e.g., `gpt-5.2`, `gpt-4.1`, `o3`, `claude-opus-4-6`, `gemini-3-pro-preview`, `deepseek-chat`) and excludes removed duplicate `openai/gpt-4o-mini` + deprecated `gemini-2.0-flash`.
- C-03 (AC-3, unit): `ModelRef::parse("deepseek/deepseek-chat")` resolves to `Provider::OpenAi` with model `deepseek-chat`; auth key candidates include DeepSeek env vars.
- C-04 (AC-4, functional): local template + script can source user-provided `.tau/provider-keys.env` and execute provider smoke commands without printing secret values.
- C-05 (AC-5, regression): `cargo test -p tau-provider` and `cargo test -p tau-ai` pass for touched behavior.

## Success Metrics

- P0 provider/catalog slice from roadmap is implemented with test evidence.
- User can place local provider keys in one documented location and run live validation commands safely.
