# Plan 3810: Repo-Aware Code Verifier Planning

## Approach

Extend the existing derived-verifier path with a repo-aware code verifier:

1. Run `cargo metadata --no-deps --format-version 1` in the target repo.
2. Extract package names from metadata and keep only argv-safe names.
3. Resolve a package only if the issue text references one of those package
   names as a token.
4. Resolve a test/filter only from a quoted or backticked argv-safe token that
   looks like a test filter (`spec`, `test`, `::`, or `regression_`).
5. Derive exactly `cargo test -p <package> <test-filter>`.
6. Persist that command in intake and feed it into the durable job loop only
   when edit/provider authority exists and intake is otherwise ready.

Everything else remains fail-closed with the existing required-authority
contract.

## Affected Files

- `crates/tau-runtime/src/autonomous_coding_jobs_runtime.rs`
- `crates/tau-runtime/src/autonomous_coding_jobs_runtime/verifier_derivation.rs`
- `crates/tau-runtime/src/autonomous_coding_jobs_runtime/tests/repo_aware_verifier.rs`
- `docs/guides/autonomous-coding-jobs.md`
- `specs/3810-repo-aware-code-verifiers/*`

## Risks

- Risk: package names containing `cli` skew issue classification.
  Mitigation: explicit Rust/test signals win over the CLI substring path.
- Risk: unsafe verifier command construction.
  Mitigation: package and filter are restricted to safe argv tokens and no shell
  parsing is used.
- Risk: cargo metadata is unavailable in non-Rust repos.
  Mitigation: derivation returns no command and the existing blocked intake path
  remains authoritative.

## Interfaces

No CLI or schema changes. This adds a narrow behavior to
`AutonomousCodingJobRuntime::run_issue_to_merge` and the persisted
`AutonomousCodingVerifierPlan`.
