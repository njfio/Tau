# Plan #2480

## Approach
1. Create milestone/index and child spec artifacts before code changes.
2. Deliver behavior through #2482 with test-first workflow in #2483.
3. Validate via scoped tests, clippy, mutation testing, and live validation script.

## Risks / Mitigations
- Risk: expanding template variables could blur scope.
  Mitigation: keep aliases startup-safe and default unresolved runtime data to deterministic empty strings.

## Interfaces / Contracts
- No external API contract changes in this orchestration issue.
- Child issue #2482 updates internal rendering contract in `tau-onboarding`.
