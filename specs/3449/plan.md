# Plan: Issue #3449 - M298 workspace artifact hygiene and ignore policy

Status: Implemented

## Approach
1. Enumerate current untracked artifacts and classify transient vs authored.
2. Update `.gitignore` for recurring transient outputs.
3. Remove current transient directories/files with scoped `rm -rf` commands.
4. Verify with `git status --short --branch` and targeted existence checks.

## Risks / Mitigations
- Risk: deleting user-authored artifacts by mistake.
  - Mitigation: restrict cleanup to known generated paths only.
- Risk: over-broad ignore patterns hiding useful files.
  - Mitigation: keep ignore patterns specific to observed transient outputs.

## Interfaces / Contracts
- Contract is local repository hygiene: ignore patterns + deterministic cleanup evidence.
