# Plan: Issue #3543 - GPT-5 default model migration for runtime entrypoints

Status: Implemented

## Approach
1. Add RED coverage for new default-model expectations in touched tests.
2. Update model defaults in:
   - `crates/tau-cli/src/cli_args.rs`,
   - `crates/tau-tui/src/main.rs`,
   - `scripts/run/tau-unified.sh`.
3. Update launcher/TUI tests for new default model strings.
4. Replace operator-facing `openai/gpt-4o-mini` guidance in README and docs
   with `openai/gpt-5.2`.
5. Run focused test and lint/format checks for touched surfaces.

## Affected Modules
- `crates/tau-cli/src/cli_args.rs`
- `crates/tau-tui/src/main.rs`
- `crates/tau-tui/tests/tui_demo_smoke.rs`
- `scripts/run/tau-unified.sh`
- `scripts/run/test-tau-unified.sh`
- `scripts/dev/provider-keys.env.example`
- `scripts/dev/provider-live-smoke.sh`
- `README.md`
- `docs/guides/*.md` (model snippet updates)
- `specs/milestones/m321/index.md`
- `specs/3543/spec.md`
- `specs/3543/plan.md`
- `specs/3543/tasks.md`

## Risks / Mitigations
- Risk: default-model change may break tests that assert old defaults.
  - Mitigation: run focused suites (`tau-cli`, `tau-tui`, launcher tests) and
    update assertions explicitly.
- Risk: docs drift across many guides.
  - Mitigation: repo-wide targeted replacement for operator docs and verify via
    grep.
- Risk: provider-mode mismatch for OAuth paths.
  - Mitigation: keep model override explicit and document GPT-5.3 as override,
    while defaulting to catalog-backed `openai/gpt-5.2`.

## Interfaces / Contracts
- `tau-rs --model` default: `openai/gpt-5.2`
- `tau-tui agent --model` default: `openai/gpt-5.2`
- `scripts/run/tau-unified.sh` model default/help: `openai/gpt-5.2`

## ADR
No ADR required (default configuration uplift; no architectural/protocol
changes).

## Execution Summary
1. Added RED assertion in `tau-tui` parser tests for new GPT-5 default.
2. Migrated runtime defaults from `openai/gpt-4o-mini` to `openai/gpt-5.2` in:
   - `crates/tau-cli/src/cli_args.rs`,
   - `crates/tau-tui/src/main.rs`,
   - `scripts/run/tau-unified.sh`.
3. Updated conformance tests and launcher contract script for new model ids:
   - `crates/tau-tui/src/main.rs`,
   - `crates/tau-tui/tests/tui_demo_smoke.rs`,
   - `scripts/run/test-tau-unified.sh`.
4. Updated README + operator guides to remove `openai/gpt-4o-mini` baseline
   guidance in runnable snippets.

## Verification Notes
- RED evidence:
  - `cargo test -p tau-tui regression_spec_c06_agent_mode_defaults_to_gpt5_baseline`
    failed pre-implementation with:
    - left=`openai/gpt-4o-mini`, right=`openai/gpt-5.2`.
- GREEN evidence:
  - `cargo test -p tau-tui` passed.
  - `bash scripts/run/test-tau-unified.sh` passed.
  - `cargo test -p tau-cli --lib` passed.
- REGRESSION evidence:
  - `cargo fmt --check` passed.
  - `cargo clippy -p tau-tui -p tau-cli -- -D warnings` passed.
  - `bash -n scripts/run/tau-unified.sh scripts/run/test-tau-unified.sh` passed.
