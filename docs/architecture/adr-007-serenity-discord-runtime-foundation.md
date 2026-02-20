# ADR-007: Serenity Dependency and Discord Runtime Foundation Boundary

## Context
G10 in `tasks/spacebot-comparison.md` retained two unresolved foundation items:

1. Add `serenity` as a workspace dependency.
2. Create a dedicated Discord runtime crate/module boundary.

Discord behavior already exists in `tau-multi-channel`, but the repository lacked
a first-class Discord runtime foundation crate and an explicit dependency contract
for Discord-native runtime evolution.

## Decision
1. Add `serenity` as a workspace dependency with a minimal explicit feature set:
   `client`, `gateway`, `model`, and `rustls_backend`.
2. Create a new crate, `tau-discord-runtime`, to host Discord runtime bootstrap
   contracts and validation surfaces.
3. Keep this slice non-invasive: no behavior migration away from existing
   `tau-multi-channel` flows in this step; the crate is a foundation boundary for
   subsequent Discord runtime increments.

## Consequences
### Positive
- Discord-native runtime work now has a dedicated crate boundary.
- Dependency governance is explicit and reviewable.
- Existing Discord behavior remains stable while architecture evolves incrementally.

### Negative
- Workspace dependency graph expands with `serenity`.
- Additional crate introduces maintenance overhead.

### Neutral / Follow-on
- Future milestones can migrate Discord runtime logic into `tau-discord-runtime`
  incrementally without forcing a single large refactor.
- Additional ADRs may be added for deeper runtime ownership and gateway/connector
  division decisions.
