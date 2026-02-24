# Spec: Issue #3547 - Full legacy mini-model reference purge

Status: Implemented

## Problem Statement
Legacy mini-model references remain across repository artifacts after
default migration, causing inconsistency between operator policy and practical
guidance. We need a complete, verifiable purge and a regression guard.

## Scope
In scope:
- Replace active references to legacy mini-model aliases across
  source, scripts, docs, tests, and fixtures.
- Keep model values aligned to GPT-5 defaults (`openai/gpt-5.2`) or codex
  provider requirements (`openai/gpt-5.3-codex`) where applicable.
- Add conformance automation to prevent legacy model string regressions.

Out of scope:
- Provider architecture changes.
- Rewriting historical external artifacts outside repository source control.
- Non-OpenAI model policy redesign.

## Acceptance Criteria
### AC-1 Legacy model strings are fully removed from tracked code and docs
Given repository tracked files,
when scanning for known legacy mini-model aliases,
then no matches remain.

### AC-2 Runtime-facing defaults and runnable examples are GPT-5 aligned
Given CLI/TUI/unified launcher and their user-facing docs/scripts,
when run or inspected without explicit overrides,
then model examples/defaults use GPT-5 family identifiers.

### AC-3 Regression guard exists for future changes
Given CI/local verification flow,
when conformance checks run,
then a dedicated check fails if legacy mini-model aliases reappear in tracked
sources.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Conformance | repo tracked files | run legacy-model scan | zero matches |
| C-02 | AC-2 | Functional | default runtime entrypoints and scripts | execute/inspect defaults | GPT-5 model ids only |
| C-03 | AC-3 | Regression | verification script | intentionally include legacy token (RED), then remove (GREEN) | guard fails then passes |

## Success Metrics / Observable Signals
- Repository scans over tracked files return no legacy mini-model aliases.
- Existing launcher/runtime tests continue to pass with updated references.
- New guard test script passes and can be used in CI/pre-PR checks.

## AC Verification
| AC | Result | Evidence |
| --- | --- | --- |
| AC-1 | ✅ | `bash scripts/run/test-model-reference-policy.sh` passes after migration; tracked-file scan reports zero legacy mini-model aliases. |
| AC-2 | ✅ | Runtime/docs/tests updated to GPT-5 family refs across `crates/*`, `scripts/*`, `specs/*`, and `tasks/*`; verified with `cargo test --workspace --no-run`, `cargo test -p tau-provider`, `cargo test -p tau-onboarding`, `cargo test -p tau-gateway`, `cargo test -p tau-coding-agent --test cli_integration auth_provider`, `cargo test -p tau-tui`, and `cargo test -p tau-cli --lib`. |
| AC-3 | ✅ | Added `scripts/run/test-model-reference-policy.sh`; RED run failed with 397 matches, GREEN run passes and is rerunnable as regression guard. |
