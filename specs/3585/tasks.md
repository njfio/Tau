# Tasks: Issue #3585 - Codex auth runtime rejects unsupported models and stops TUI hangs

- [x] T1 (RED): add provider and TUI regressions for unsupported Codex-auth model failure.
- [x] T2 (GREEN): implement the Codex-auth unsupported-model guard and actionable TUI-facing error propagation.
- [x] T3 (VERIFY): rerun scoped provider/TUI tests, formatting, clippy, and lockfile checks.
- [x] T4 (CLOSE): path-limited commit, push, and close #3585 with verification evidence.

## Tier Mapping
- Unit: provider compatibility predicate covers unsupported and supported model examples.
- Functional: supported Codex-auth model path remains accepted.
- Regression: unsupported Codex-auth `openai/gpt-5.2` fails closed with actionable text.
- Integration: TUI-facing failure text is rendered or converted through the existing interactive error surface.
- Property: N/A - finite compatibility examples, no parser/invariant surface added.
- Contract/DbC: N/A - no formal contracts in this crate path.
- Snapshot: N/A - no stable structured output snapshots.
- Fuzz: N/A - no untrusted parser surface added.
- Mutation: N/A for this narrow guard; covered by direct negative/positive regressions.
- Performance: N/A - validation is constant-time string/auth-mode matching.
