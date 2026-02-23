# M176 - CLI Args Module Split Phase 1 (Runtime Feature Flags)

Status: Completed

## Context
`crates/tau-cli/src/cli_args.rs` remains a high-churn hotspot at 3,788 lines. This milestone starts the next decomposition wave by extracting the post-`execution_domain` runtime/deployment flag declarations into dedicated source artifacts while preserving clap CLI behavior.

## Scope
- Keep `Cli` as the external parse surface.
- Extract the runtime feature tail block (events/rpc/deployment domain flags) from root file.
- Preserve all flag names, env bindings, defaults, and help behavior.
- Validate with tau-cli and tau-coding-agent focused regression suites.

## Linked Issues
- Epic: #2990
- Story: #2991
- Task: #2992

## Closeout
- Phase 1 delivered via issue `#2992`, extracting runtime/deployment flag artifacts while preserving clap contract.
- Governance closure delivered via issue `#3408`, formally closing GitHub milestone `M176` on 2026-02-23.

## Success Signals
- Linked epic/story/task are all closed.
- Milestone `M176` is closed in GitHub milestone metadata.
