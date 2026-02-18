# Plan #2483

## Approach
- Add C-01..C-04 tests that assert minijinja alias behavior and fallback compatibility.
- Execute scoped suite before implementation and record failure output.
- Implement #2482, rerun suite, and record passing output.

## Risks / Mitigations
- Risk: existing passing tests hide missing RED signal.
  Mitigation: reference new enum/fields/behavior in tests before implementation to force RED.
