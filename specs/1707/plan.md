# Issue 1707 Plan

Status: Reviewed

## Approach

1. Refresh `m23-rustdoc-marker-count` artifacts using existing counting script.
2. Add `rustdoc-marker-threshold-verify.sh` to compare baseline/current JSON and
   produce machine-readable + markdown gate artifacts.
3. Add targeted contract test script for verification output semantics.
4. Publish gate report showing threshold status and per-crate deltas.

## Affected Areas

- `scripts/dev/rustdoc-marker-threshold-verify.sh` (new)
- `scripts/dev/test-rustdoc-marker-threshold-verify.sh` (new)
- `tasks/reports/m23-rustdoc-marker-count*.{json,md}`
- `tasks/reports/m23-rustdoc-marker-threshold-verify.{json,md}`
- `specs/1707/*`

## Risks And Mitigations

- Risk: baseline source ambiguity.
  - Mitigation: persist baseline snapshot artifact and record paths in output.
- Risk: threshold unmet could be misread as script failure.
  - Mitigation: separate script execution success from gate status field.

## ADR

No architecture/dependency/protocol change. ADR not required.
