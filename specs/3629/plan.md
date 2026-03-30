# Plan: Issue #3629 - Close live_rl_runtime oversized-file blocker

1. Create a fresh milestone container and live GitHub issue hierarchy for the
   blocker instead of attaching new work to the closed M25 decomposition wave.
2. Define a single implementation story that owns:
   - reducing `crates/tau-coding-agent/src/live_rl_runtime.rs` below 4000 lines;
   - preserving existing live RL runtime behavior;
   - restoring the oversized-file guard without a new exemption.
3. Keep the decomposition work isolated from unrelated PRs, especially #3628,
   so the blocker remains explicit and reviewable.

## Risks / Mitigations
- Risk: the blocker gets hidden as “someone else’s CI problem.”
  - Mitigation: create an explicit active milestone and story with tracked exit
    criteria.
- Risk: future implementation shortcuts to a policy exemption instead of a real
  split.
  - Mitigation: make no-new-exemption behavior part of the story acceptance
    criteria.

## Interface / Contract Notes
- No code or API contract changes are introduced by this epic-planning slice.
- The epic exists to establish the binding delivery path for the actual story.
