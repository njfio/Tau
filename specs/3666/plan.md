# Plan: Issue #3666 - Teach CLI provider adapters to emit textual tool-call payloads

## Approach
1. Change the prompt contract for tool-enabled CLI provider requests so the
   model knows it may return plain text or textual `tool_calls` JSON.
2. Remove the misleading context-only wording from Codex, Claude, and Gemini
   CLI adapters and replace it with the same executable contract text.
3. Add a Codex regression that proves the contract can produce promoted tool
   calls and prompt-level tests for Claude and Gemini.

## Proposed Design
### Prompt contract
- Keep plain assistant text support for non-tool responses.
- When tools exist, render:
  - a response contract stating the model may return plain text or a textual
    JSON object
  - explicit tool-call schema guidance
  - instruction to use listed tool names exactly
  - tool descriptions plus parameter schemas

### Compatibility Assessment
```yaml
implementation_strategy:
  task: "3666"
  change_surface:
    - symbol: "CLI provider prompt contract for tool-enabled requests"
      location: "crates/tau-provider/src/{codex_cli_client,claude_cli_client,gemini_cli_client}.rs"
      change_type: "modification"
      current: "tool-enabled requests are rendered as plain-text-only with tools described as context only"
      proposed: "tool-enabled requests explicitly support textual tool-call JSON emission"
      compatibility: "safe"
      reason: "preserves plain-text responses while enabling tool execution on CLI-backed model paths"
  overall_compatibility: "safe"
  approach:
    strategy: "Align CLI adapter prompt contracts with Tau textual tool-call promotion"
    steps:
      - "update Codex, Claude, and Gemini prompt rendering for tool-enabled requests"
      - "add Codex regression proving tool-call promotion under the new contract"
      - "add prompt assertions for Claude and Gemini alignment"
    version_impact: "none"
```

## Risks / Mitigations
- Risk: prompt wording change degrades plain-text answers.
  Mitigation: preserve explicit plain-text fallback wording and keep existing
  plain-text parsing tests green.
- Risk: adapters drift again over time.
  Mitigation: add prompt-level tests for all three CLI adapters using the same
  contract phrases.

## Verification
- `cargo test -p tau-provider codex_cli_client -- --test-threads=1`
- `cargo test -p tau-provider claude_cli_client -- --test-threads=1`
- `cargo test -p tau-provider gemini_cli_client -- --test-threads=1`
- `rustfmt --check --edition 2021 crates/tau-provider/src/codex_cli_client.rs crates/tau-provider/src/claude_cli_client.rs crates/tau-provider/src/gemini_cli_client.rs`
