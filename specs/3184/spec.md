# Spec: Issue #3184 - deterministic directory-path rejection for browser DID report writer

Status: Implemented

## Problem Statement
When `write_browser_did_init_report` is given a directory path, it currently fails indirectly via filesystem write errors. This produces non-deterministic OS-level error text and weakens fail-closed contract clarity.

## Scope
In scope:
- Add deterministic pre-write guard rejecting directory output paths.
- Add conformance test verifying explicit error message for directory-path input.
- Preserve successful nested-file write behavior.

Out of scope:
- Report format changes.
- New dependencies.
- Cross-crate API changes.

## Acceptance Criteria
### AC-1 Directory output path is rejected with deterministic message
Given an existing directory path,
when `write_browser_did_init_report` is called with that path,
then it fails with a deterministic `report output path must be a file path` message.

### AC-2 Valid file output behavior remains unchanged
Given a valid nested file output path,
when `write_browser_did_init_report` is called,
then it writes JSON with trailing newline and preserves existing output contract.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
|---|---|---|---|---|---|
| C-01 | AC-1 | Unit/Conformance | output path points to existing directory | write report | deterministic directory-path rejection error |
| C-02 | AC-2 | Integration/Conformance | nested valid file path | write report | output file exists, parseable JSON, trailing newline |

## Success Metrics / Observable Signals
- `cargo test -p kamn-sdk spec_3184 -- --test-threads=1`
- `cargo test -p kamn-sdk`
- `cargo fmt --check`
- `cargo clippy -p kamn-sdk -- -D warnings`
