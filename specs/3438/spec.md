# Spec: Issue #3438 - Fix Docker smoke build missing workspace member

Status: Implemented

## Problem Statement
`scripts/dev/docker-image-smoke.sh` currently fails when building `tau-coding-agent` because `Dockerfile` copies `Cargo.toml` and `crates/`, but the workspace also includes `tests/integration` as a member. Cargo fails with a missing manifest error for `/workspace/tests/integration/Cargo.toml`.

## Scope
In scope:
- Ensure Docker build context includes workspace members required by `cargo build --release -p tau-coding-agent`.
- Add/extend deterministic packaging contract checks so this regression fails fast before image build.
- Re-run Docker smoke build verification.

Out of scope:
- Changing runtime entrypoint behavior.
- Expanding image contents beyond what's required for workspace resolution.
- CI workflow redesign.

## Acceptance Criteria
### AC-1 Docker workspace resolution succeeds
Given the repository Dockerfile and workspace manifest,
when `cargo build --release -p tau-coding-agent` runs in the Docker builder stage,
then Cargo resolves all workspace members without missing manifest errors.

### AC-2 Packaging contract fails closed on missing workspace member copy
Given the packaging contract script,
when required workspace-member copy statements are absent,
then `scripts/dev/test-docker-image-packaging.sh` exits non-zero with a deterministic error.

### AC-3 Docker smoke build completes successfully
Given a healthy Docker daemon and network,
when `scripts/dev/docker-image-smoke.sh --tag tau-coding-agent:local-smoke` runs,
then image build and `tau-coding-agent --help` smoke validation both pass.

## Conformance Cases
| Case | AC | Tier | Given | When | Then |
| --- | --- | --- | --- | --- | --- |
| C-01 | AC-1 | Functional | current workspace membership includes `tests/integration` | run Docker builder compile step | no missing workspace-member manifest error |
| C-02 | AC-2 | Regression | packaging contract script | remove required copy line and run script | deterministic non-zero failure with workspace-member message |
| C-03 | AC-2 | Regression | updated packaging contract script | run script against fixed Dockerfile | script exits zero |
| C-04 | AC-3 | Conformance | Docker daemon available | run docker smoke script | build succeeds and runtime help check passes |

## Success Metrics / Observable Signals
- Docker smoke no longer fails on `/workspace/tests/integration/Cargo.toml` missing.
- Packaging contract script enforces workspace-member copy requirement.
- No regressions in formatting/lint/tests for touched scope.

## Implementation Evidence
### RED
- `bash scripts/dev/test-docker-image-packaging.sh`
  - `error: Dockerfile must copy tests/integration for workspace member resolution`
- `bash scripts/dev/docker-image-smoke.sh --tag tau-coding-agent:local-smoke`
  - `failed to read /workspace/tests/integration/Cargo.toml` (before `COPY tests/integration` fix)
  - then `ort_sys` linker failure on Bookworm (`undefined reference to __isoc23_*` / `__cxa_call_terminate`)

### GREEN
- `bash scripts/dev/test-docker-image-packaging.sh`
  - `docker image packaging contract tests passed`
- `bash scripts/dev/docker-image-smoke.sh --tag tau-coding-agent:local-smoke`
  - `docker image smoke summary: status=pass image=tau-coding-agent:local-smoke`

### Hygiene
- `cargo fmt --all -- --check`
