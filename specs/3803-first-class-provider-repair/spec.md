Status: Reviewed

# Spec 3803: First-Class OpenRouter Repair Adapter

## Problem

The durable autonomous coding job loop can already call a provider repair command,
parse strict JSON edit payloads or unified diffs, apply the repair, and rerun
verifiers. The live-provider boundary is still an external shell adapter. Tau
needs a built-in OpenRouter-compatible adapter so provider repair is a product
capability, not a proof script convention.

## Scope

In scope:
- A `tau-autonomous-coding-job openrouter-repair-adapter` subcommand.
- A `--provider-repair-openrouter` shortcut on job submission and issue-to-merge.
- `.env` and process environment resolution for OpenRouter API key, model, and
  API base.
- Strict validation against Tau's existing provider repair JSON/edit/diff
  contract before stdout is emitted.
- Provider call metadata stored next to the durable repair context.
- Deterministic mock OpenRouter proof through the real `issue-to-merge` loop.

Out of scope:
- Non-OpenRouter provider adapters.
- Provider-side streaming.
- Automatic merge without explicit `--allow-auto-merge`.
- Bypassing branch protection or using `gh --admin`.

## Acceptance Criteria

### AC-1: Built-in adapter resolves provider configuration

Given a repair job configured with `--provider-repair-openrouter`
When no external provider repair command is supplied
Then Tau must invoke its own `openrouter-repair-adapter` subcommand
And resolve model/API key/API base from explicit flags, process env, or `.env`.

### AC-2: Provider output is fail-closed

Given the adapter receives provider text
When the text is not a valid Tau provider repair payload
Then the adapter must fail before printing repair JSON
And the durable loop must record the failed/rejected repair evidence.

### AC-3: Valid provider repair runs through the durable loop

Given an issue-to-merge job with verifier authority and built-in OpenRouter
repair enabled
When the first verifier run fails
Then Tau must call the built-in adapter, apply the returned multi-file edit set,
rerun the verifier, and produce PR-ready or draft-PR evidence.

### AC-4: Provider call metadata is persisted without secrets

Given a provider repair adapter call
When the call succeeds or fails after reading the repair context
Then Tau must store metadata including provider, model, API base, auth source,
usage/finish reason when available, output hash, edit count, and error summary
when relevant
And the API key must never be written.

### AC-5: Draft PR remains the safe default, auto-merge remains explicit

Given an issue-to-merge command omits `--pr-mode`
When the job reaches PR publication
Then draft mode is the default safe output
And auto-merge still requires explicit `--allow-auto-merge`.

## Conformance Cases

- C-01 maps AC-1/AC-3/AC-4: mock OpenRouter server receives a JSON-mode chat
  request from `issue-to-merge`, returns a two-file edit payload, Tau applies the
  repair, records metadata, and reaches `pr_ready`.
- C-02 maps AC-1: adapter model resolution reads
  `TAU_PROVIDER_PROOF_MODEL` from an explicit `.env` file.
- C-03 maps AC-1: `openrouter/<model>` is stripped before the model id is sent
  to the OpenRouter-compatible API.
- C-04 maps AC-2: fenced provider JSON is extracted and validated against the
  existing Tau repair parser before stdout emission.
- C-05 maps AC-5: CLI defaults for submit and issue-to-merge use draft PR mode
  unless overridden.

## Success Signals

- `scripts/dev/test-openrouter-repair-adapter.sh` passes.
- `scripts/dev/test-autonomous-coding-gauntlet.sh` includes the built-in adapter
  case.
- Focused Rust tests for the adapter pass under `tau_autonomous_coding_job`.
