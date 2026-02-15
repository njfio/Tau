# Issue 1651 Plan

Status: Reviewed

## Approach

1. Add script `scripts/dev/rustdoc-marker-count.sh`:
   - scan `crates/*/src/**/*.rs`
   - count lines beginning with `///` or `//!`
   - emit summary to stdout and optional JSON/Markdown artifacts
2. Add script contract test `scripts/dev/test-rustdoc-marker-count.sh` with:
   - deterministic fixture counts
   - JSON schema field checks
   - unknown option / missing path regression checks
3. Update `docs/guides/doc-density-scorecard.md` with usage and artifact paths.
4. Generate and check in baseline M23 marker-count artifacts.

## Affected Areas

- `scripts/dev/rustdoc-marker-count.sh` (new)
- `scripts/dev/test-rustdoc-marker-count.sh` (new)
- `docs/guides/doc-density-scorecard.md` (update)
- `tasks/reports/m23-rustdoc-marker-count.json` (new generated artifact)
- `tasks/reports/m23-rustdoc-marker-count.md` (new generated artifact)

## Output Contract

JSON fields:

- `schema_version`
- `generated_at`
- `repo_root`
- `scan_root`
- `total_markers`
- `crates[]` (`crate`, `markers`, `files_scanned`)

## Risks And Mitigations

- Risk: fixture tests diverge from script scanning rules
  - Mitigation: keep regex and file selection simple and explicit in test.
- Risk: confusion with existing doc-density percent tool
  - Mitigation: docs update clarifies marker-count vs public-API density roles.

## ADR

No dependency/protocol changes. ADR not required.
