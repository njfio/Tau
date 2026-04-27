# ADR 0004: Experimental direct OpenAI Responses OAuth/session transport

- **Status**: Accepted
- **Date**: 2026-04-27
- **Deciders**: user + Gyre SE agent

## Context

Issue #3676 asks Tau to stop depending on the Codex CLI subprocess as the only
OpenAI oauth/session-style transport during agentic runtime work. The current
subprocess path adds prompt-contract translation, timeout behavior, and
empty-response failure modes that have been harming Ralph-loop recovery. Tau
already has a direct OpenAI-compatible HTTP client for supported API-key auth.
Official OpenAI Responses documentation describes `POST /v1/responses` with
bearer-token authentication for API keys and `stream: true` for server-sent
event streaming. Direct use of Codex/OpenAI OAuth or session tokens against the
public Responses endpoint is not a documented supported contract.

## Decision

Add an explicit experimental opt-in path that may route OpenAI OAuth/session
credentials through Tau's direct OpenAI Responses HTTP transport when a usable
bearer token is resolved, while keeping API-key behavior unchanged and retaining
Codex CLI fallback whenever the experiment is disabled or cannot resolve direct
auth material.

The compatibility boundary is `crates/tau-provider`: provider selection may use
the direct HTTP client only when the new experimental flag is enabled and the
resolved credential can be wired as bearer authentication. Default behavior must
continue to prefer the existing supported paths.

## Consequences

### Positive

- Ralph-loop recovery can avoid an unnecessary Codex CLI subprocess hop when
  the operator explicitly enables the experiment.
- Supported API-key HTTP behavior remains the stable default.
- The unsupported OAuth/session direct path is reversible because the existing
  Codex CLI fallback stays available.
- Focused provider-selection tests can prove the opt-in boundary without making
  live OpenAI calls.

### Negative

- OAuth/session direct transport relies on behavior that OpenAI does not expose
  as a documented public API contract.
- Operators must understand that failures in this path may be provider-side or
  credential-contract changes rather than Tau regressions.
- The provider-selection code gains another branch that must stay covered by
  tests.

### Neutral

- This ADR does not remove Codex CLI support.
- This ADR does not change Anthropic, Google, or OpenRouter transport policy.
- Live OAuth/session validation remains outside this local unit-test stage.

## Alternatives considered

1. **Keep OAuth/session auth exclusively on Codex CLI.** Rejected because it
   leaves the known subprocess timeout and empty-response failure mode in the
   Ralph-loop recovery path.
2. **Switch OAuth/session auth to direct HTTP by default.** Rejected because the
   external API contract is undocumented and should not surprise operators.
3. **Remove Codex CLI fallback after adding direct transport.** Rejected because
   the experimental path must be reversible and because unsupported credential
   material can fail to work with the public endpoint at any time.

## References

- #3676
- `specs/3676/spec.md`
- `crates/tau-provider/src/client.rs`
- `crates/tau-provider/src/types.rs`
- OpenAI Responses API reference: `https://developers.openai.com/api/reference/resources/responses/methods/create`