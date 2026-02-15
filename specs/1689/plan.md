# Issue 1689 Plan

Status: Reviewed

## Approach

1. Capture RED gap list for missing `//!` module headers.
2. Add concise, file-specific headers grouped by module role:
   - startup/runtime orchestration
   - commands/profile/tools policies
   - channel/transport/rpc diagnostics
3. Run scoped checks:
   - header scan (GREEN)
   - `cargo test -p tau-coding-agent`
   - docs link regression check

## Affected Areas

- `crates/tau-coding-agent/src/*.rs` (targeted files)
- `specs/1689/*`

## Risks And Mitigations

- Risk: high file count increases merge friction.
  - Mitigation: header-only changes, no behavior edits.
- Risk: boilerplate docs with low signal.
  - Mitigation: each header references concrete contract/failure semantics.

## ADR

No architecture/dependency/protocol change. ADR not required.
