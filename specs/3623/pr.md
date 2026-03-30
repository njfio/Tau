# Review Handoff: Issue #3623

## Git State
- Current branch: `feat/radical-simplification-self-improvement`
- Repository baseline branch: `master`
- Current `HEAD` is `49` commits ahead of `master`
- Working tree contains unrelated tracked and untracked changes outside this issue

Because the checkout is not on a clean issue branch and the worktree is mixed,
do not create a PR or commit by blindly staging everything.

## Safe Scope
Stage only these paths for the runtime-integrity closure wave:

```bash
git add \
  crates/tau-agent-core/src/lib.rs \
  crates/tau-agent-core/src/tests/action_history.rs \
  crates/tau-tools/src/mcp_server_runtime.rs \
  crates/tau-ops/src/mcp_sdk.rs \
  crates/tau-orchestrator/src/plan_executor.rs \
  specs/3623 \
  specs/3624 \
  specs/3625 \
  specs/3626 \
  specs/3627 \
  specs/milestones/m329
```

Then confirm the staged scope before committing:

```bash
git diff --cached --stat
git diff --cached --name-only
```

## Branch Suggestion
If this work is moved onto a clean branch or worktree rooted from `master`, use:

```bash
git checkout -b codex/issue-3623-runtime-integrity-closure
```

## Commit Suggestion
```bash
git commit -m "fix(runtime): close runtime integrity gaps (#3623)"
```

## PR Title
`fix(runtime): 3623 close runtime integrity gaps across persistence, MCP, skills, and orchestration`

## PR Description
This change closes the runtime-integrity gaps tracked in `#3623`. It fixes
prompt-path action-history persistence, records real tool turn/latency data,
stops MCP lifecycle tools from returning fake-success responses when no runtime
backing exists, routes MCP skills operations through `tau-skills`, and aligns
`plan_executor` documentation with its actual reporting/deadlock-analysis
surface.

Implementation touched the agent core, MCP runtime/docs, and orchestrator
documentation/tests. In `crates/tau-agent-core/src/lib.rs`, prompt entrypoints
now persist action history consistently and tool records capture measured
latency plus the real turn index. In
`crates/tau-tools/src/mcp_server_runtime.rs` and
`crates/tau-ops/src/mcp_sdk.rs`, training and agent lifecycle MCP tools now
return explicit runtime-unavailable contracts, and the MCP skills list/info/install
path now delegates to `tau-skills` instead of maintaining a weaker parallel
filesystem implementation. In
`crates/tau-orchestrator/src/plan_executor.rs`, module/item docs now describe
reporting and deadlock analysis only, with regression coverage locking that
truth-in-advertising wording.

Scoped verification passed on every touched runtime surface:
- `rustfmt --check --edition 2021` on the touched files
- `cargo test -p tau-agent-core`
- `cargo test -p tau-integration-tests --test agent_tool_memory_roundtrip`
- `cargo test -p tau-tools mcp_server_runtime -- --test-threads=1`
- `cargo test -p tau-ops mcp_sdk -- --test-threads=1`
- `cargo test -p tau-skills install_skills -- --nocapture`
- `cargo test -p tau-skills skills_lockfile -- --nocapture`
- `cargo test -p tau-orchestrator plan_executor -- --test-threads=1`

Repo-level verification is still blocked by pre-existing issues outside this
scope:
- `cargo fmt --check` fails on unrelated workspace formatting drift
- `cargo clippy -- -D warnings` is blocked by existing deprecated-item warnings
  from `tau-extensions`

This pull request resolves `#3623`.
