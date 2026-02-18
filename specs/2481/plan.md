# Plan #2481

## Approach
- Reuse existing template source resolution/reporting from M81.
- Swap only rendering internals to minijinja and add alias context values.
- Preserve reason-code and source selection behavior.

## Risks / Mitigations
- Risk: output drift from whitespace differences.
  Mitigation: assert stable conformance substrings and preserve current fallback behavior.

## Interfaces / Contracts
- `compose_startup_system_prompt_with_report` signature remains unchanged.
- Template variable contract expands to include alias names while keeping legacy names valid.
