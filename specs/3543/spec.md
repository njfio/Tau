# Spec: Issue #3543 - GPT-5 default model migration for runtime entrypoints

Status: Implemented

## Problem Statement
Operators want GPT-5 generation models as the default runtime baseline, but
current defaults still use `openai/gpt-5.2`. This causes undesired model
selection in unified TUI flow and CLI startup unless overridden manually.

## Scope
In scope:
- Update default model id to `openai/gpt-5.2` for:
  - `tau-cli` `--model` default,
  - `tau-tui agent` mode default,
  - `tau-unified.sh` model default.
- Update tests that encode default model contracts.
- Update README and operator docs to remove `openai/gpt-5.2` as recommended
  baseline.

Out of scope:
- Removing catalog compatibility entries for older models.
- Forcing model pinning that blocks explicit user override.
- Provider capability redesign.

## Acceptance Criteria
### AC-1 Runtime defaults use GPT-5 baseline model
Given no explicit model argument is passed,
when operators invoke CLI/TUI/launcher entrypoints,
then default model resolves to `openai/gpt-5.2`.

### AC-2 Test contracts reflect new default model semantics
Given targeted test suites,
when they run after migration,
then default-model assertions pass with `openai/gpt-5.2`.

### AC-3 Operator docs no longer recommend gpt-5.2 defaults
Given README and operator guides,
when reviewing startup examples,
then default/recommended model snippets use GPT-5 generation model ids.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Unit | CLI parser defaults | parse no model arg | default model is `openai/gpt-5.2` |
| C-02 | AC-1 | Functional | `tau-unified.sh` default model | invoke `up`/`tui` without model | command/help default references `openai/gpt-5.2` |
| C-03 | AC-2 | Conformance | `tau-tui` tests | run suite | agent default assertions pass |
| C-04 | AC-3 | Functional | docs content | inspect snippets | no recommended `openai/gpt-5.2` defaults remain |

## Success Metrics / Observable Signals
- New unified TUI sessions default to GPT-5 baseline model without manual flags.
- CLI default model aligns with operator policy.
- Docs and tests consistently reflect GPT-5 baseline.

## AC Verification
| AC | Result | Evidence |
| --- | --- | --- |
| AC-1 | ✅ | `crates/tau-cli/src/cli_args.rs`, `crates/tau-tui/src/main.rs`, and `scripts/run/tau-unified.sh` defaults now resolve to `openai/gpt-5.2`. |
| AC-2 | ✅ | `cargo test -p tau-tui`; `cargo test -p tau-cli --lib`; `bash scripts/run/test-tau-unified.sh`. |
| AC-3 | ✅ | `README.md` and `docs/guides/*.md` operator snippets updated from `openai/gpt-5.2` to `openai/gpt-5.2`. |
