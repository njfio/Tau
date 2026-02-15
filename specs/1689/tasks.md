# Issue 1689 Tasks

Status: Implemented

## Ordered Tasks

T1 (tests-first): capture RED gap list for missing `//!` headers in `tau-coding-agent`.

T2: add module headers for runtime/startup orchestration files.

T3: add module headers for command/tool/profile/policy files.

T4: add module headers for channel/transport/rpc/failure-diagnostic files.

T5: run scoped checks (`cargo test -p tau-coding-agent`, docs checks).

## Tier Mapping

- Functional: targeted module headers present
- Conformance: runtime/command/failure contracts documented
- Regression: `tau-coding-agent` tests + docs checks pass
