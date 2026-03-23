# Plan: Issue #3618 - Interactive skill auto-selection

Status: Completed

## Approach
1. Add a shared skill-selection helper in `tau-skills` that can:
   - merge multiple skill directories into one catalog
   - score prompt relevance against skill metadata/content
   - union explicit selections with prompt-driven auto-selections
2. Add a repo-shipped bundled skill under `skills/` for Phaser/web-game build work so the root checkout has a deterministic match target.
3. Use the shared selector in the coding-agent interactive runtime before each turn by recomposing and replacing the system prompt for the current prompt.
4. Use the same selection heuristics in the gateway openresponses execution path, because the graphical TUI submits through `/v1/responses` rather than the CLI REPL loop.
5. Mirror the same selector in the graphical TUI before submission so the operator sees the same active skills the runtime will use.
6. Keep selection conservative: only concrete build/create/game prompts should activate the bundled web-game skill.

## Affected Modules
- `crates/tau-skills`
- `crates/tau-onboarding`
- `crates/tau-coding-agent`
- `crates/tau-gateway`
- `crates/tau-tui`
- `skills/`

## Risks / Mitigations
- Risk: overly broad matching activates the web-game skill for unrelated prompts.
  - Mitigation: conservative keyword/token scoring and regression tests for non-matches.
- Risk: TUI and runtime drift on selected skills.
  - Mitigation: both paths call the same shared helper and tests assert visible names.
- Risk: replacing the system prompt per turn clobbers template hot-reload behavior.
  - Mitigation: recompute from current startup composition each turn rather than appending ad hoc text.
- Risk: gateway-backed graphical TUI diverges from CLI interactive behavior.
  - Mitigation: use the same selector in the openresponses execution path and keep the TUI visibility path read-only.
- Risk: empty `.tau/skills` still leaves no useful behavior.
  - Mitigation: merge bundled repo `skills/` as a fallback catalog source.

## Interfaces / Contracts
- New shared skill-selection report returns:
  - effective selected skills
  - explicit selection names
  - auto-selected names
- Interactive runtime reuses that report to replace the system prompt for the current turn.
- Interactive TUI status/chat surfaces display the same selected skill names for the current turn.
- Gateway openresponses preloads startup-available skills into `GatewayOpenResponsesServerConfig` so prompt augmentation stays cycle-free.

## ADR
No ADR required. This is a bounded runtime/TUI behavior correction using existing skills infrastructure.
