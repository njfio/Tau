# M223 - kamn-sdk Fail-Closed Report Output Path Validation

Status: In Progress

## Context
`write_browser_did_init_report` relies on filesystem errors when the target path is a directory. The resulting error is platform-dependent and not a deterministic fail-closed contract.

## Scope
- Enforce explicit rejection when output path is a directory.
- Add conformance coverage for deterministic directory-path rejection behavior.
- Preserve existing nested-file write behavior.

## Linked Issues
- Epic: #3182
- Story: #3183
- Task: #3184

## Success Signals
- `cargo test -p kamn-sdk spec_3184 -- --test-threads=1`
- `cargo test -p kamn-sdk`
- `cargo fmt --check`
- `cargo clippy -p kamn-sdk -- -D warnings`
