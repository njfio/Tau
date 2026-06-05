# Plan: Issue #3764 - Agent Canvas proof loop stores comparison evidence

## Approach

Keep the implementation in the existing proof automation. The script already
submits `/ops/chat/send`, checks route recovery behavior, verifies that an HTML
artifact exists, fetches `/ops/chat`, and validates Agent Canvas v2 markers. Add a
small loop layer around those checks:

1. Capture the first route inspection and artifact hash as the `before`
   iteration.
2. Apply a deterministic proof-only artifact fix that adds a durable marker,
   a canvas, an input target, and a console success signal.
3. Fetch `/ops/chat` again and run the same route contract checks.
4. Persist a comparison section with before/after hashes, marker visibility,
   stable route-contract status, and rerun status.

The shell test remains the RED/GREEN contract for CI. It uses a fake `curl`
gateway and a temporary artifact so the proof-loop JSON shape is validated
without credentials or a browser dependency.

## Affected Modules

- `scripts/dev/ops-chat-canvas-proof.sh`
- `scripts/dev/test-ops-chat-canvas-proof.sh`
- `specs/3764-agent-canvas-proof-loop/*`
- `tasks/reports/ops-chat-canvas-proof-loop.json` if live proof is run and
  checked in as evidence

## Risks and Mitigations

- Risk: Shell-only proof could overstate browser runtime evidence.
  Mitigation: Persist diagnostics as contract/capability markers, not as actual
  browser pixel values, unless a live browser proof is separately recorded.
- Risk: The targeted fix could mask a missing artifact.
  Mitigation: Keep the existing file check before applying the fix.
- Risk: JSON assembled in shell could become invalid.
  Mitigation: Test with `jq` when available and assert stable fields in the
  shell regression.
- Risk: The issue number collides with existing `specs/3764/`.
  Mitigation: Use the explicit conflict-safe path
  `specs/3764-agent-canvas-proof-loop/` and call that out in the issue/PR.

## Verification

Run the shell regression first, then focused static checks. If a local gateway
with the necessary tool path is available, run the proof script against it and
store the resulting comparison evidence under `tasks/reports/`.
