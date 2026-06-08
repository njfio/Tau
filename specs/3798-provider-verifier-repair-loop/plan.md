# Plan: Provider Verifier Repair Loop

GitHub Issue: #3798
Status: Implemented

## Approach

Extend the existing provider-backed harness rather than adding a parallel agent:

- Add a `provider_repair_attempts` CLI setting with a small bounded default.
- Track each provider attempt in the JSON report.
- Keep the first attempt prompt compatible with the existing proof path.
- On verifier failure without a hard blocked reason, collect:
  - failed verifier command argv,
  - failed stdout/stderr snippets,
  - current changed files,
  - current git diff snippet.
- Send that repair context to the provider prompt.
- Apply the returned edit set with `CodingMissionRunner::resume`.
- Stop immediately when the mission reaches PR-ready, when provider parsing
  fails, when the verifier is hard-blocked, or when attempts are exhausted.

## Affected Modules

- `crates/tau-coding-agent/src/bin/tau_live_coding_loop_harness.rs`
- `scripts/dev/test-provider-verifier-repair-loop.sh`
- `README.md`
- `docs/guides/canonical-product-proof.md`
- `specs/3798-provider-verifier-repair-loop/*`

## Risks And Mitigations

- Risk: unbounded retry behavior hides bad model output. Mitigation: bounded
  attempts and explicit exhausted-repair fail-closed reason.
- Risk: repair prompt leaks secrets from command output. Mitigation: reuse
  redaction before embedding stdout/stderr/diff snippets.
- Risk: existing provider proof report consumers expect a single `provider`.
  Mitigation: keep `provider` as the final attempt and add `provider_attempts`
  as a backward-compatible field.

## Interfaces

- CLI: `--provider-repair-attempts <n>` controls additional provider repair
  attempts after the first provider edit.
- CLI: repeated `--mock-provider-response` values apply only to their matching
  attempt; later attempts call the configured provider.
- Report: `provider_attempts: ProviderProofReport[]`.
- Provider prompt: repair attempts include failed verifier and diff context.

## ADR

No ADR required: this is bounded harness behavior and does not change provider
protocols, transport semantics, schemas outside the harness report, or
dependencies.
