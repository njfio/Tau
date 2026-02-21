# M189 - KAMN Core Coverage Hardening

## Context
`kamn-core` has foundational DID generation tests but limited explicit coverage for malformed identifier segments and canonical normalization behavior that callers rely on.

## Scope
- Add conformance-focused malformed-input and normalization tests for `build_browser_did_identity`.
- Implement the minimal runtime guard required if RED tests expose malformed DID acceptance.
- Keep changes scoped to `crates/kamn-core` and issue-linked spec artifacts.

## Linked Issues
- Epic: #3042
- Story: #3043
- Task: #3044
