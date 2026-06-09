# Plan 3809: Derived Docs Verifier for Issue-to-Merge

## Approach

Keep verifier derivation deliberately narrow. `issue-to-merge` may auto-derive a
verifier only when all of these are true:

- The request omits explicit verifier commands.
- The issue text looks like docs/readme/documentation/guide work.
- The issue contains a quoted or backticked marker that is safe as one argv
  token: ASCII alphanumeric plus `_`, `-`, `.`, or `:`, length 3..96, not
  starting with `-`.
- Edit or provider repair authority is present.
- Intake classification is still `ready_to_run`.

When these hold, Tau uses:

- `git diff --check`
- `grep -n <token> README.md` for README issues, or
  `grep -R -n <token> docs` for docs issues

Everything else continues through the existing blocked intake path.

## Affected Files

- `crates/tau-runtime/src/autonomous_coding_jobs_runtime.rs`
- `docs/guides/autonomous-coding-jobs.md`
- `specs/3809-derived-docs-verifier/*`

## Risks

- Risk: Tau overstates this as arbitrary verifier generation.
  Mitigation: docs and tests state this is single-token docs verification only.
- Risk: unsafe tokens become shell injection.
  Mitigation: commands are split by argv, not shell, and the accepted marker is
  limited to safe single-token characters.
- Risk: broad/vague issues sneak through.
  Mitigation: intake classification still runs and must produce
  `ready_to_run`.

## Interfaces

No CLI flag changes. This is a runtime behavior improvement for
`tau-autonomous-coding-job issue-to-merge` when `verifier_commands` are empty.
