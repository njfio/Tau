## Linked Issue

Closes #<issue-id>

## Summary of behavior changes

- <change 1>
- <change 2>

## Risks and compatibility notes

- <risk or compatibility impact>

## Impacted live surfaces

Select every impacted surface:

- [ ] voice
- [ ] browser automation
- [ ] dashboard
- [ ] custom command
- [ ] memory
- [ ] none (no live-surface impact)

## Mandatory live-run evidence (required when any surface above is checked)

For each impacted surface, include at least one concrete evidence link:

- CI workflow/job/artifact URL, or
- repository artifact path (for example `.tau/live-run-unified/report.json`)

| Surface | Evidence link(s) | Result summary |
| --- | --- | --- |
| voice | <link/path> | <pass/fail + key numbers> |
| browser automation | <link/path> | <pass/fail + key numbers> |
| dashboard | <link/path> | <pass/fail + key numbers> |
| custom command | <link/path> | <pass/fail + key numbers> |
| memory | <link/path> | <pass/fail + key numbers> |

If a surface is not impacted, write `n/a`.

## Validation matrix evidence (required)

- Unit:
  - `<command>`
  - `<result>`
- Functional:
  - `<command>`
  - `<result>`
- Integration:
  - `<command>`
  - `<result>`
- Regression:
  - `<command>`
  - `<result>`
- Formatting:
  - `cargo fmt --all -- --check`
  - `<result or reason not run>`
- Lint:
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `<result or reason not run>`

## Final validation package + sign-off

- [ ] Attached final validation package or linked package artifact location.
- [ ] Included go/no-go summary with approver name and timestamp.
- [ ] If rollout-sensitive, completed release sign-off checklist.

References:

- `docs/guides/release-signoff-checklist.md`
- `docs/guides/final-validation-package.md`
