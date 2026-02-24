# Plan: Issue #3547 - Full legacy mini-model reference purge

Status: Implemented

## Approach
1. Add RED conformance guard that scans tracked files for legacy model strings
   and fails if any are present.
2. Perform repo-wide replacement for active legacy mini-model references, selecting
   `openai/gpt-5.2` or `openai/gpt-5.3-codex` based on call path semantics.
3. Manually review edge files (specs/tasks/changelogs/tests) to avoid invalid
   substitutions.
4. Run focused verification for changed modules and scripts.
5. Record AC/tier evidence and update spec status to `Implemented`.

## Affected Modules
- `crates/*` where tests/examples still carry legacy model strings
- `scripts/` runtime/test/dev helpers
- `docs/` and `README.md`
- `specs/` where active execution guidance references old model ids
- New conformance guard script under `scripts/run/`

## Risks / Mitigations
- Risk: over-replacing historical context text.
  - Mitigation: run targeted replacements, then inspect diff and revert
    semantically required historical mentions.
- Risk: replacing codex-only model usage with non-codex model.
  - Mitigation: keep codex pathways on `openai/gpt-5.3-codex`.
- Risk: regression from broad text edits.
  - Mitigation: run targeted crate tests and script checks after replacements.

## Interfaces / Contracts
- No wire/protocol changes.
- Contract update: repository conformance check enforces absence of
  legacy mini-model tokens in tracked sources.

## ADR
No ADR required (policy/documentation/default-string consistency; no
architecture or dependency changes).

## Execution Summary
1. Added conformance guard script:
   - `scripts/run/test-model-reference-policy.sh`
2. Captured RED evidence:
   - Guard failed with `397` tracked matches before replacement.
3. Executed controlled repository-wide replacement from legacy mini-model tokens
   to GPT-5 family references, then corrected semantic regressions in:
   - `crates/tau-provider/src/model_catalog.rs`
   - `crates/tau-coding-agent/tests/cli_integration/auth_provider.rs`
4. Re-ran verification and stabilized all touched high-impact suites.

## Verification Notes
- Formatting/lint:
  - `cargo fmt --check`
  - `cargo clippy -p tau-provider -p tau-coding-agent -p tau-onboarding -p tau-gateway -p tau-tui -p tau-cli --all-targets -- -D warnings`
- Conformance/scripts:
  - `bash scripts/run/test-model-reference-policy.sh`
  - `bash scripts/run/test-tau-unified.sh`
- Compile sweep:
  - `cargo test --workspace --no-run`
- Executable tests:
  - `cargo test -p tau-provider`
  - `cargo test -p tau-onboarding`
  - `cargo test -p tau-gateway`
  - `cargo test -p tau-coding-agent --test cli_integration auth_provider`
  - `cargo test -p tau-tui`
  - `cargo test -p tau-cli --lib`
