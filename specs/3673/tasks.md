# Tasks: Force Tool-Required Retry Turns In Ralph-Loop Recovery

- [x] T1. Add failing/targeted assertions for required tool choice on gateway retries and CLI prompt rendering.
- [x] T2. Implement agent-level tool-choice override support.
- [x] T3. Apply required tool choice in gateway action retries.
- [x] T4. Honor required tool choice in CLI provider prompt contracts.
- [x] T5. Run scoped gateway/provider verification.

## Evidence refresh — 2026-04-27

- Added provider prompt-contract regressions proving `ToolChoice::Required` renders a hard textual tool-call requirement for Codex, Claude, and Gemini CLI adapters.
- Added gateway regression proving generic mutation recovery retries fall back to `ToolChoice::Required` when the concrete `write` heuristic does not apply.
- Verified scoped provider/gateway regressions plus the full `gateway_openresponses` gateway test filter before closeout.
