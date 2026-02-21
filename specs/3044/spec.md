# Spec: Issue #3044 - KAMN core malformed-input and normalization hardening

Status: Reviewed

## Problem Statement
`kamn-core` currently validates character classes for `network` and `subject`, but it does not explicitly reject empty dotted segments (for example `edge..tau`), which can render malformed DID components. Normalization behavior also needs explicit conformance coverage for trimmed/lowercase canonicalization.

## Acceptance Criteria

### AC-1 Reject malformed dotted identifier segments
Given a DID identity request where `network` or `subject` contains empty dotted segments,
When `build_browser_did_identity` is called,
Then it fails with a deterministic validation error and does not return an identity.

### AC-2 Preserve canonical normalization behavior
Given valid `network` and `subject` inputs with mixed case and surrounding whitespace,
When `build_browser_did_identity` is called,
Then returned `network` and `subject` are canonicalized to lowercase trimmed values.

### AC-3 Maintain stable key DID contracts
Given a valid key-method request,
When `build_browser_did_identity` succeeds,
Then `did` starts with `did:key:z`, `key_id` equals `{did}#primary`, and fingerprint/proof fields remain deterministic for equivalent normalized inputs.

### AC-4 Verification gates remain green
Given the test/runtime updates,
When running validation,
Then targeted crate tests plus fmt/check/clippy pass.

## Scope

### In Scope
- `crates/kamn-core/src/lib.rs`
- `specs/milestones/m189/index.md`
- `specs/3044/*`

### Out of Scope
- Changes to `kamn-sdk` public interfaces.
- DID method expansion beyond current `key`/`web`.

## Conformance Cases
- C-01 (AC-1): `network=edge..tau` is rejected with an empty-segment validation error.
- C-02 (AC-1): `subject=agent..primary` is rejected with an empty-segment validation error.
- C-03 (AC-2): mixed-case + padded `network/subject` are normalized to lowercase trimmed values.
- C-04 (AC-3): key-method DID/key_id contract and deterministic outputs for equivalent normalized inputs.
- C-05 (AC-4): validation command set passes.

## Success Metrics / Observable Signals
- `cargo test -p kamn-core spec_c01_rejects_network_with_empty_dot_segment -- --nocapture`
- `cargo test -p kamn-core spec_c02_rejects_subject_with_empty_dot_segment -- --nocapture`
- `cargo test -p kamn-core spec_c03_normalizes_network_and_subject_to_canonical_lowercase -- --nocapture`
- `cargo test -p kamn-core spec_c04_key_method_outputs_are_stable_for_equivalent_normalized_inputs -- --nocapture`
- `cargo test -p kamn-core -- --nocapture`
- `cargo fmt --check`
- `cargo clippy -p kamn-core -- -D warnings`
- `cargo check -q`

## Approval Gate
P1 scope: spec authored/reviewed by agent; implementation proceeds and is flagged for human review in PR.
