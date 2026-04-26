# Product Proof Release Checklist

This checklist prepares the product-proof readiness release without performing it. Do not tag or push without explicit approval.

## Release Intent

Recommended bump: minor.

The release-worthy change set gives operators and reviewers a canonical local product proof:

- `scripts/dev/prove-tau-product.sh --check` for non-runtime contract validation.
- `scripts/dev/prove-tau-product.sh --run` for live lifecycle proof.
- `scripts/dev/prove-tau-product.sh --run --report <path>` for machine-readable JSON evidence.
- `scripts/dev/prove-tau-product.sh --run --webchat-smoke --report <path>` for opt-in `/webchat` readiness proof.

## Pre-Approval Checks

Run these before asking for release execution approval:

```bash
scripts/dev/prove-tau-product.sh --check
scripts/dev/test-prove-tau-product.sh
cargo fmt --check
git diff --quiet -- Cargo.toml Cargo.lock
```

Expected result: all commands pass and root Cargo files remain unchanged during plan-only preparation.

## Approval Boundary

Before any release execution, get explicit approval for all of the following:

- Target version.
- Whether to bump `Cargo.toml` from workspace `0.1.0` to the chosen minor version.
- Whether to create a release commit.
- Whether to create a git tag.
- Whether to push the branch and tags to `origin`.
- Whether to publish a GitHub release.

## Execution Steps After Approval

Do not run this section during plan-only work.

1. Move the relevant `[Unreleased]` CHANGELOG entries into the approved version section.
2. Update root `Cargo.toml` `[workspace.package] version` to the approved target version.
3. Run `cargo metadata --no-deps --format-version 1` to confirm workspace metadata is valid.
4. Run product-proof gates:

```bash
scripts/dev/prove-tau-product.sh --check
scripts/dev/test-prove-tau-product.sh
cargo fmt --check
```

5. Review `git diff` for only intended release metadata changes.
6. Commit the release metadata.
7. Create the approved tag.
8. Push the commit and tag only after final confirmation.
9. Publish GitHub release notes from `docs/solutions/release-notes/product-proof-readiness.md` only after final confirmation.

## Rollback Notes

If a release metadata change is prepared but not pushed, reset the release commit through normal git review workflow. If a tag has already been created locally but not pushed, delete only the local tag after explicit confirmation. If a tag has been pushed, treat rollback as a separate release-management decision.
