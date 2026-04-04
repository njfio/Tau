# Plan: Direct OpenAI Responses Transport With Experimental OAuth/Session Mode

## Compatibility Assessment

implementation_strategy:
  task: "3676"
  change_surface:
    - symbol: "tau_provider::build_provider_client"
      location: "crates/tau-provider/src/client.rs"
      change_type: "modification"
      current: "OpenAI oauth/session auth prefers Codex CLI backend"
      proposed: "OpenAI oauth/session auth may choose direct OpenAI HTTP transport when an explicit experimental flag is enabled"
      compatibility: "caution"
      reason: "Behavior changes only for explicit opt-in experimental mode; default path remains unchanged"
    - symbol: "tau_cli::Cli openai auth flags"
      location: "crates/tau-cli/src/cli_args.rs"
      change_type: "additive"
      current: "OpenAI auth mode and Codex backend flags"
      proposed: "New experimental direct transport flag for oauth/session auth"
      compatibility: "safe"
      reason: "Purely additive CLI/env flag"
  overall_compatibility: "caution"
  approach:
    strategy: "Direct implementation behind explicit opt-in flag with fallback"
    steps:
      - "Add additive CLI/env flag for experimental direct oauth/session transport"
      - "Teach provider selection to attempt direct OpenAI HTTP first only when the flag is enabled"
      - "Preserve Codex CLI fallback for disabled/unsupported/unresolved cases"
      - "Add focused selection tests so the compatibility boundary is explicit"
    version_impact: "none; opt-in experimental behavior only"

## Implementation Approach

1. Add a new CLI/env flag for OpenAI experimental direct oauth/session
   transport.
2. Inspect resolved OpenAI credentials and identify when Tau has a bearer token
   that can be passed directly to `build_openai_http_client`.
3. Update `build_provider_client` so:
   - API-key path stays as-is
   - oauth/session auth with experimental flag enabled prefers direct HTTP
   - fallback remains Codex CLI when direct auth material is unavailable
4. Add focused tests around provider selection and auth wiring.

## Risks

- Direct use of oauth/session bearer tokens against public OpenAI HTTP endpoints
  may not be a supported contract.
  - Mitigation: explicit experimental opt-in, retain fallback, document risk in
    spec and final handoff.
- Credential resolution may return material intended only for CLI/backend flows.
  - Mitigation: only attempt direct HTTP when the resolved credential is usable
    as a bearer token; otherwise fallback.
- CLI/api-key path regression.
  - Mitigation: preserve default selection logic and cover it with focused tests.

## Verification

- provider selection tests in `tau-provider`
- any existing OpenAI HTTP client tests needed to validate auth/wiring
- scoped runtime rebuild after implementation
