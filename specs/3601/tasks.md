# Tasks: Issue #3601 - Align CLI backend timeout with request timeout budget

- [x] T1 (RED): add provider regressions proving CLI backend timeout selection is floored by `request_timeout_ms` and larger backend-specific values are preserved.
- [x] T2 (GREEN): wire Codex, Codex app-server, Claude, and Gemini backend construction through the shared timeout selector.
- [x] T3 (VERIFY): rerun scoped provider adapter tests, formatting, clippy, and lockfile checks.
- [x] T4 (CLOSE): path-limited commit, push, and close #3601 with verification evidence.

## Tier Mapping
- Unit: timeout selector covers backend-smaller, backend-larger, and zero-value normalization.
- Functional: existing Codex, Claude, and Gemini CLI adapter filters remain green.
- Regression: provider client construction no longer permits CLI backend timeout below the request timeout budget.
- Integration: Codex mock CLI can sleep past the old backend timeout but complete within the caller budget.
- Property: N/A - finite timeout selector examples are sufficient.
- Contract/DbC: N/A - no formal contract framework in this path.
- Snapshot: N/A - no serialized output snapshots.
- Fuzz: N/A - no parser surface added.
- Mutation: N/A for this arithmetic helper; direct boundary cases cover the behavior.
- Performance: N/A - constant-time timeout selection only.
