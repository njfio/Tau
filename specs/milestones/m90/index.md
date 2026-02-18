# M90 - Spacebot G14 SendFileTool Contract Closure

- Milestone: https://github.com/njfio/Tau/milestone/90
- Epic: #2523
- Story: #2524
- Task: #2525
- Subtask: #2526

## Goal
Close the remaining G14 gap by introducing a first-class `send_file` built-in tool contract and wiring turn-suppression/diagnostic handling so file-delivery intent is explicit and testable.

## In Scope
- `SendFileTool` in `tau-tools` + built-in registry exposure.
- `tau-agent-core` extraction/suppression behavior for successful `send_file` tool results.
- `tau-coding-agent` outbound event payload metadata for send-file directives.
- Conformance + regression + mutation + live-validation evidence.

## Out of Scope
- Net-new transport backends.
- Expanding existing outbound adapter file-delivery transport matrix beyond current implementation.
