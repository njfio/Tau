# M321 - GPT-5 default model uplift

Status: Active

## Context
Runtime entrypoints still default to `openai/gpt-4o-mini`, which is no longer
the desired operator baseline. M321 updates default model behavior to GPT-5
generation model ids across CLI/TUI/launcher paths and aligns operator docs.

Primary sources:
- `crates/tau-cli/src/cli_args.rs`
- `crates/tau-tui/src/main.rs`
- `scripts/run/tau-unified.sh`
- `README.md`
- `docs/guides/`

## Issue Hierarchy
- Epic: #3541
- Story: #3542
- Task: #3543

## Scope
- Change default model id from `openai/gpt-4o-mini` to `openai/gpt-5.2` for:
  - `tau-cli` startup default,
  - `tau-tui agent` mode default,
  - `scripts/run/tau-unified.sh` default.
- Update conformance/unit tests reflecting default model strings.
- Update operator-facing docs/snippets that still promote
  `openai/gpt-4o-mini`.

## Exit Criteria
- `specs/3543/spec.md` is `Implemented` with AC evidence.
- CLI/TUI/launcher defaults resolve to GPT-5 baseline model.
- Focused tests and script checks pass.
- Docs no longer present `openai/gpt-4o-mini` as recommended default.
