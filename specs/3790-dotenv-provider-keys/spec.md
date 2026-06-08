# Spec: Repo `.env` provider key fallback

Status: Accepted

## Problem

Local provider-backed proof runs should be able to use a gitignored repo-root `.env` file for OpenRouter/DeepSeek API keys without requiring operators to export the same variables in every shell. The fallback must not commit secrets and must not override explicitly exported process environment values.

## Scope

In:
- Load provider API-key candidates from a nearby `.env` file when the process environment does not define the variable.
- Support standard `KEY=value`, optional `export KEY=value`, and simple quoted values.
- Preserve existing provider auth behavior when `.env` is absent.
- Keep `.env` ignored and untracked.

Out:
- Persisting secrets to tracked files.
- Adding a dotenv dependency.
- Loading arbitrary runtime configuration beyond provider key candidates.

## Acceptance Criteria

AC-1: Given a repo-root `.env` contains `OPENROUTER_API_KEY`, when Tau resolves OpenRouter API-key auth from a subcommand run inside the repo tree, then the key is available as the OpenRouter credential.

AC-2: Given both the process environment and `.env` define a provider key, when Tau resolves the credential, then the process environment value wins.

AC-3: Given `.env` is absent, empty, or contains only comments/unsupported keys, when Tau resolves provider credentials, then existing missing-key behavior is unchanged.

AC-4: Given `.env` contains secret-like values, when provider-backed proof reports are written, then reports do not include the raw key.

## Conformance Cases

C-01 maps AC-1: `OPENROUTER_API_KEY=` absent from process env, repo `.env` contains a non-empty value, `provider_api_key_candidates` includes an `OPENROUTER_API_KEY` candidate with that value.

C-02 maps AC-2: process env contains `OPENROUTER_API_KEY=env-value`, `.env` contains `OPENROUTER_API_KEY=file-value`, resolver returns `env-value`.

C-03 maps AC-3: temporary cwd without `.env` returns the same absent candidates as before.

C-04 maps AC-4: OpenRouter DeepSeek V4 provider-backed proof report contains provider metadata and hashes, not secret-shaped tokens.

## Success Signals

- Unit tests cover `.env` fallback, process-env precedence, and absent file behavior.
- Provider-backed autonomous coding-loop proof passes with `.env` as the credential source.
