# Issue 3788 Plan

## Approach

Extend `tau_live_coding_loop_harness` with a `provider-success` mode. The mode
will build a provider client through Tau's existing `tau-ai` clients, prompt for
strict JSON edit instructions, parse the response into
`CodingMissionControlledEdit`, and then call the existing `CodingMissionRunner`
path. The runner remains responsible for verifier red/green evidence, commit
creation, checkpoint state, and PR-ready bundle generation.

Provider evidence is stored as sanitized metadata: provider, model, response
text byte count, finish reason, token usage, parse status, and reason code. API
keys and raw environment values are never printed or written.

## Affected Modules

- `crates/tau-coding-agent/src/bin/tau_live_coding_loop_harness.rs`
- `scripts/dev/test-provider-backed-autonomous-coding-loop.sh`
- `README.md`
- `docs/guides/canonical-product-proof.md`
- `specs/3788-provider-backed-coding-loop-proof/`

## Risks

- Live provider models can drift, rate-limit, or return malformed output.
- CI cannot require third-party credentials.
- A direct provider mode can accidentally become a side proof if it bypasses the
  mission runner.

## Mitigations

- Keep provider-backed live proof opt-in through `TAU_LIVE_PROVIDER_PROOF=1`.
- Add deterministic malformed-output coverage without live credentials.
- Feed parsed provider edits through `CodingMissionRunner` only.
- Fail closed with structured report reasons before any commit or PR-ready
  bundle when provider prerequisites are missing or invalid.

## Interfaces

Adds CLI flags to the harness binary:
- `--mode provider-success`
- `--provider-model <provider/model>`
- `--provider-api-base <url>`
- `--mock-provider-response <json-or-text>`

The report schema remains versioned and gains an optional `provider` section.
No production wire protocol changes.
