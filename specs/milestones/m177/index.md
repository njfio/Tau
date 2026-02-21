# M177 - Panic and Unsafe Audit Guardrails

## Context
Quality review flagged growth in raw `panic!` and `unsafe` keyword counts. Most occurrences are test-only, but the current ad-hoc checks do not separate production-risk paths from test harnesses. This milestone adds deterministic audit output and production-path guardrail evidence.

## Scope
- Add a reproducible audit script for `panic!` and `unsafe` usage with path-class breakdown.
- Validate production targets remain clean via panic/unsafe clippy guardrail commands.
- Publish contract artifacts and evidence for repeatable review.

## Linked Issues
- Epic: #2994
- Story: #2995
- Task: #2996
