# Spec: Direct OpenAI Responses Transport With Experimental OAuth/Session Mode

Status: Implemented
Issue: #3676

## Problem Statement

Tau currently routes OpenAI `oauth-token` and session-style auth through the
Codex CLI backend. That subprocess transport is adding prompt-contract
translation, timeout behavior, and empty-response failures that are harming the
gateway Ralph-loop path. Tau already has a direct OpenAI HTTP client for API-key
auth, but not for experimental oauth/session direct transport.

The user explicitly wants Tau to try direct transport with the Codex/OpenAI
session OAuth token even if that path is not a documented supported public API
contract. The implementation must preserve the existing supported API-key path
and retain Codex CLI fallback.

## Scope

In scope:
- add a direct OpenAI Responses transport selection path for OpenAI
  `oauth-token` and `session-token` auth modes
- gate unsupported direct oauth/session transport behind an explicit experimental
  flag
- preserve the existing supported direct API-key HTTP path
- preserve Codex CLI fallback when the experimental direct transport is
  disabled, unavailable, or credential resolution cannot produce a bearer token
- add provider-selection and auth-wiring tests for the new path

Out of scope:
- removing the Codex CLI backend
- changing Anthropic or Google auth transport selection
- changing Tau’s outer Ralph-loop logic
- guaranteeing that OpenAI will support direct use of session OAuth tokens

## Acceptance Criteria

### AC-1
Given OpenAI provider selection with `oauth-token` or `session-token` auth
When the new experimental direct transport flag is enabled and Tau resolves a
bearer token credential
Then Tau chooses the direct OpenAI HTTP Responses transport instead of the Codex
CLI backend.

### AC-2
Given OpenAI provider selection with supported API-key auth
When Tau builds the provider client
Then the existing direct OpenAI HTTP path remains unchanged.

### AC-3
Given OpenAI provider selection with `oauth-token` or `session-token` auth
When the experimental direct transport flag is disabled or credential resolution
cannot provide a direct bearer token
Then Tau falls back to the existing Codex CLI backend behavior.

### AC-4
Given the experimental direct oauth/session transport path
When Tau builds the direct OpenAI HTTP client
Then it uses bearer-token authentication and routes Codex-capable models through
Responses endpoint behavior already covered by `tau-ai`.

## Conformance Cases

- C-01: `oauth-token` + experimental direct flag + resolved bearer token =>
  direct OpenAI HTTP client selected. Maps to AC-1. Tier: Conformance.
- C-02: `session-token` + experimental direct flag + resolved bearer token =>
  direct OpenAI HTTP client selected. Maps to AC-1. Tier: Conformance.
- C-03: API-key auth still selects the existing OpenAI HTTP client. Maps to
  AC-2. Tier: Conformance.
- C-04: `oauth-token` with experimental direct flag disabled falls back to
  Codex CLI backend. Maps to AC-3. Tier: Conformance.
- C-05: `oauth-token` with experimental direct flag enabled but no direct
  bearer credential falls back to Codex CLI backend. Maps to AC-3. Tier:
  Conformance.

## Success Metrics

- Tau can be launched in an explicit experimental mode that attempts direct
  OpenAI Responses transport for oauth/session auth.
- Existing supported API-key behavior is unchanged.
- The new path is isolated behind explicit opt-in and test coverage.

## Verification Evidence

- `cargo test -p tau-provider openai_experimental_direct_transport -- --test-threads=1`
- `cargo test -p tau-provider codex -- --test-threads=1`
- `cargo fmt --check`
- `cargo clippy -p tau-provider --tests --no-deps -- -D warnings`

## External API Verification

- Official OpenAI Responses documentation describes `POST /v1/responses` at
  `https://api.openai.com/v1/responses` with `Authorization: Bearer $OPENAI_API_KEY`.
- Official documentation describes `stream: true` as server-sent event streaming.
- Direct use of Codex/OpenAI OAuth or session tokens for this endpoint is not a
  documented public API contract; this stage must remain explicit opt-in and
  preserve Codex CLI fallback.
