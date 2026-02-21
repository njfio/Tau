# Spec: Issue #3048 - KAMN SDK malformed-input propagation and report contract hardening

Status: Reviewed

## Problem Statement
`kamn-sdk` wraps `kamn-core` DID construction for browser-facing callers, but current coverage does not explicitly verify malformed-input propagation contracts and durable report persistence semantics. SDK errors should remain deterministic and context-rich for operators.

## Acceptance Criteria

### AC-1 Propagate malformed-input failures with SDK context
Given invalid DID initialization inputs rejected by `kamn-core`,
When `initialize_browser_did` is called,
Then the SDK returns an error that includes both SDK context and source validation details.

### AC-2 Preserve deterministic normalized equivalence
Given semantically equivalent `network/subject` inputs that differ only by case/whitespace,
When `initialize_browser_did` is called,
Then generated identity outputs are equivalent and deterministic.

### AC-3 Preserve report persistence contract
Given a nested output path for `write_browser_did_init_report`,
When writing a report,
Then parent directories are created, JSON is persisted with trailing newline, and payload round-trips.

### AC-4 Verification gates remain green
Given the test/runtime updates,
When running validation,
Then targeted crate tests plus fmt/clippy/check pass.

## Scope

### In Scope
- `crates/kamn-sdk/src/lib.rs`
- `specs/milestones/m190/index.md`
- `specs/3048/*`

### Out of Scope
- DID rendering algorithm changes in `kamn-core`.
- Changes to gateway runtime endpoints.

## Conformance Cases
- C-01 (AC-1): malformed dotted-segment network input returns context-rich SDK error.
- C-02 (AC-2): normalized-equivalent requests produce identical DID/fingerprint/proof values.
- C-03 (AC-3): report writing creates nested directories and persists valid JSON with trailing newline.
- C-04 (AC-4): validation command set passes.

## Success Metrics / Observable Signals
- `cargo test -p kamn-sdk spec_c01_initialize_browser_did_propagates_malformed_input_with_sdk_context -- --nocapture`
- `cargo test -p kamn-sdk spec_c02_initialize_browser_did_is_deterministic_for_normalized_equivalent_inputs -- --nocapture`
- `cargo test -p kamn-sdk spec_c03_write_browser_did_init_report_persists_nested_json_with_trailing_newline -- --nocapture`
- `cargo test -p kamn-sdk -- --nocapture --test-threads=1`
- `cargo fmt --check`
- `cargo clippy -p kamn-sdk -- -D warnings`
- `cargo check -q`

## Approval Gate
P1 scope: spec authored/reviewed by agent; implementation proceeds and is flagged for human review in PR.
