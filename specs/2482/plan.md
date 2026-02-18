# Plan #2482

## Approach
1. Add `minijinja` to workspace dependencies and `tau-onboarding` crate.
2. Replace `render_prompt_template` manual parser with minijinja environment rendering.
3. Build a render context including both legacy keys and new alias keys:
   - legacy: `base_system_prompt`, `skills_section`, `identity_sections`, `default_system_prompt`
   - aliases: `identity`, `tools`, `memory_bulletin`, `active_workers`
4. Preserve existing fallback/report source selection paths.
5. Add C-01..C-04 tests (RED first), then implement until GREEN.

## Risks / Mitigations
- Risk: minijinja parse errors alter reason-code paths.
  Mitigation: keep fallback routing unchanged and assert explicit reason-codes in regressions.
- Risk: alias defaults unintentionally non-deterministic.
  Mitigation: use fixed startup-safe defaults (`""` for unavailable runtime fields).

## Interfaces / Contracts
- Public API unchanged: `compose_startup_system_prompt_with_report` and `StartupPromptTemplateReport`.
- Internal rendering contract expanded with alias key support.
