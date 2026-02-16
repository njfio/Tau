# Plan #2219

Status: Implemented
Spec: specs/2219/spec.md

## Approach

1. RED: add/extend unit tests for model catalog schema/entries and deepseek alias/key candidate behavior.
2. GREEN: implement schema fields, refresh built-in entries, and add deepseek alias/key wiring.
3. Implement local-safe key template + smoke validation script and update quickstart docs.
4. VERIFY: run targeted tests and formatting checks.

## Affected Modules

- `crates/tau-provider/src/model_catalog.rs`
- `crates/tau-ai/src/provider.rs`
- `crates/tau-provider/src/auth.rs`
- `crates/tau-cli/src/cli_args.rs`
- `scripts/dev/provider-keys.env.example`
- `scripts/dev/provider-live-smoke.sh`
- `docs/guides/quickstart.md`
- `tasks/resolution-roadmap.md`
- `specs/milestones/m42/index.md`
- `specs/2219/spec.md`
- `specs/2219/plan.md`
- `specs/2219/tasks.md`

## Risks and Mitigations

- Risk: large catalog refresh breaks lookup assumptions.
  - Mitigation: conformance tests for required presence/absence and lookup behavior.
- Risk: secret leakage in docs/scripts.
  - Mitigation: template-only placeholders, script redacts/masks values and avoids echoing secrets.
- Risk: deepseek alias changes unsupported provider messaging.
  - Mitigation: focused parser/error tests.

## Interfaces and Contracts

- RED tests:
  - `cargo test -p tau-provider conformance_built_in_model_catalog_refresh_contains_required_models`
  - `cargo test -p tau-provider unit_provider_api_key_candidates_include_deepseek_env_vars_for_openai`
  - `cargo test -p tau-ai parses_deepseek_as_openai_alias`
- GREEN/VERIFY:
  - `cargo test -p tau-provider`
  - `cargo test -p tau-ai`
  - `cargo fmt --check`

## ADR References

- Not required.
