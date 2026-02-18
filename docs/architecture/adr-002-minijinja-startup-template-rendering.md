# ADR-002: Adopt `minijinja` for startup prompt template rendering

## Context
`tau-onboarding` startup prompt templates were rendered by a custom placeholder parser that only supported direct token substitution and non-Spacebot variable names. This blocked G17 parity goals from `tasks/spacebot-comparison.md` and made template semantics diverge from Jinja-based operator expectations.

## Decision
Adopt `minijinja` as a workspace dependency and use it for startup prompt template rendering in `crates/tau-onboarding/src/startup_prompt_composition.rs`.

Implementation choices:
- Strict undefined behavior (`UndefinedBehavior::Strict`) to preserve fail-closed semantics.
- Keep trailing newline enabled to preserve prior output behavior.
- Provide both legacy keys and Spacebot-style startup-safe aliases:
  - legacy: `base_system_prompt`, `skills_section`, `identity_sections`, `default_system_prompt`
  - aliases: `identity`, `tools`, `memory_bulletin`, `active_workers`
- Preserve existing workspace -> builtin -> default fallback routing and diagnostics.

## Consequences
Positive:
- Startup templates now support Jinja syntax and filters.
- Operator-facing template compatibility improves without breaking existing templates.
- Error handling remains deterministic and fail-closed.

Tradeoffs:
- Adds one dependency (`minijinja`) and associated maintenance surface.
- Rendering semantics are now delegated to the templating engine, requiring conformance tests to guard behavior.
