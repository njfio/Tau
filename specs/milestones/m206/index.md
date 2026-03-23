# M206 - Interactive Skill Auto-Selection

## Context
Implements the interactive Tau runtime skill-selection correction:
- `3618` "Interactive runtime does not auto-select task-relevant skills"

for the coding-agent interactive loop, gateway `/v1/responses` path, and graphical TUI visibility.

## Linked Issues
- Task: #3618

## Scope
- Merge `.tau/skills` and bundled repo `skills/` when selecting prompt-time skills.
- Auto-select relevant skills for concrete implementation prompts while preserving explicit skill selections.
- Apply the same skill-selection behavior in the local interactive runtime and the gateway openresponses path used by the graphical TUI.
- Surface the selected skill names in the graphical TUI so operators can verify active guidance.
- Add targeted conformance and regression coverage across `tau-skills`, `tau-coding-agent`, `tau-gateway`, and `tau-tui`.

## Out of Scope
- Automatic loading of every installed skill.
- Remote skill download, trust, or registry semantics.
- Tool policy or orchestrator-routing redesign.
- Streaming/progress transport redesign.
