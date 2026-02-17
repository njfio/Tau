# Milestone M46: Production Readiness Gap Closure Wave 1

Status: In Progress

## Objective

Close the highest-impact production gaps across runtime economics, provider
integration, training execution, distribution packaging, and operations safety.

## Scope

In scope:

- Critical blockers:
  - per-session cost tracking
  - token pre-flight estimation against provider/model ceilings
  - provider prompt-caching integration
  - wiring PPO/GAE into production training execution path
- Functional completeness:
  - OpenRouter first-class provider behavior
  - PostgreSQL session backend implementation
  - onboarding guided flow completion
  - dashboard direction and implementation/consolidation decision
  - WASI preview2 migration
- Distribution deliverables:
  - Docker image
  - Homebrew formula
  - shell completions
  - systemd unit
- Testing and operations:
  - fuzz testing coverage
  - log rotation policy/runtime support

Out of scope:

- unrelated feature redesign outside listed gaps
- protocol-breaking changes without separate ADR/spec approval

## Success Signals

- Each listed gap has an accepted spec-backed implementation issue in this
  milestone.
- Critical blockers are implemented with passing conformance + regression tests.
- Distribution and ops deliverables have verifiable install/run checks.

## Issue Hierarchy

Milestone: GitHub milestone `M46 Production Readiness Gap Closure Wave 1`

Epic:

- `#2245` Epic: M46 production readiness gap closure

Stories:

- `#2246` Story: M46.1 critical runtime economics and training closure
- `#2247` Story: M46.2 functional completeness closure
- `#2248` Story: M46.3 distribution packaging closure
- `#2249` Story: M46.4 testing and operations hardening

Tasks/Subtasks:

- `#2250` Task: M46.1.1 implement critical blockers 1-4
  - `#2254` Subtask: Gap-1 per-session cost tracking
  - `#2255` Subtask: Gap-2 token pre-flight estimation
  - `#2256` Subtask: Gap-3 prompt caching support
  - `#2257` Subtask: Gap-4 wire PPO/GAE into training loop
- `#2251` Task: M46.2.1 implement functional completeness gaps 5-9
  - `#2258` Subtask: Gap-5 OpenRouter first-class provider
  - `#2259` Subtask: Gap-6 PostgreSQL session backend implementation
  - `#2260` Subtask: Gap-7 onboarding wizard guided flow
  - `#2261` Subtask: Gap-8 dashboard direction and implementation
  - `#2262` Subtask: Gap-9 WASI preview2 migration
- `#2252` Task: M46.3.1 implement distribution gaps 10-13
  - `#2263` Subtask: Gap-10 Docker image packaging
  - `#2264` Subtask: Gap-11 Homebrew formula
  - `#2265` Subtask: Gap-12 shell completions
  - `#2266` Subtask: Gap-13 systemd unit
- `#2253` Task: M46.4.1 implement testing/ops gaps 14-15
  - `#2267` Subtask: Gap-14 fuzz testing
  - `#2268` Subtask: Gap-15 log rotation
