# Plan #2525

## Approach
1. Add new `SendFileTool` with strict input validation.
2. Register tool in built-in registry and name list.
3. Parse successful send_file tool outputs in `tau-agent-core` and suppress follow-up model reply.
4. Add parsing/serialization of send-file directive metadata in `tau-coding-agent` events.
5. Add RED->GREEN conformance + regression tests and run required gates.

## Risks
- Over-broad suppression from malformed payloads.
- Potential payload schema drift between core and coding-agent.

## Mitigations
- Require explicit marker (`send_file_response` or action token) and non-empty `file_path`.
- Add regressions for malformed/error payloads.
