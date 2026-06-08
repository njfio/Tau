# Plan: Repo `.env` provider key fallback

## Approach

Implement a narrow provider-auth fallback in `tau-provider` rather than adding a global dotenv dependency. Provider API-key candidate lookup will check the real process environment first. If no non-empty process value exists, it will look for the nearest `.env` while walking upward from the current directory and read only the requested provider key.

The parser stays intentionally small: skip blank/comment lines, accept optional `export`, split on the first `=`, validate shell-style key names, trim values, and strip matching single or double quotes. It does not expand variables or execute shell syntax.

The provider-backed proof script also reads repo-root `.env` before resolving its model/auth variables, but uses the same precedence rule: exported process values win over file values.

## Affected Modules

- `crates/tau-provider/src/auth.rs`
- `crates/tau-coding-agent/src/bin/tau_live_coding_loop_harness.rs`
- `scripts/dev/test-provider-backed-autonomous-coding-loop.sh`
- `specs/3790-dotenv-provider-keys/*`
- local untracked `.env`

## Risks

- Risk: loading the wrong `.env` from an unrelated parent directory.
  - Mitigation: stop upward lookup at the nearest `.git` boundary and prefer process env.

- Risk: surprising operators by overriding exported env.
  - Mitigation: process env always wins in Rust and shell paths.

- Risk: leaking secrets in durable reports.
  - Mitigation: existing provider proof stores hashes/metadata, and validation checks for secret-shaped tokens.

## Interfaces

No public provider auth dependency changes. Existing provider key env names keep working:

- `OPENROUTER_API_KEY`
- `TAU_OPENROUTER_API_KEY`
- `OPENAI_API_KEY`
- other provider key env slots already supported by `tau-provider`

The live proof harness adds `--provider-max-tokens` with a default of `1024` so OpenRouter DeepSeek V4 can complete the JSON edit instead of truncating at the previous hardcoded cap.
