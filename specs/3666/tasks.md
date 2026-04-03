# Tasks: Issue #3666 - Teach CLI provider adapters to emit textual tool-call payloads

- [x] T1 (RED): add prompt-contract regressions for Codex, Claude, and Gemini
      CLI adapters plus a Codex end-to-end textual tool-call promotion test.
- [x] T2 (GREEN): update the three CLI adapter prompt renderers to expose the
      textual tool-call contract when tools are available.
- [x] T3 (VERIFY): run scoped tau-provider tests plus rustfmt on the touched
      files.

## Tier Mapping
- Functional: tool-enabled CLI prompts expose executable textual tool-call guidance
- Regression: Codex CLI promotes tool-call payloads under the rendered contract
