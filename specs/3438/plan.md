# Plan: Issue #3438 - Docker workspace-member packaging fix

1. Add RED regression assertion in `scripts/dev/test-docker-image-packaging.sh` requiring Dockerfile to copy `tests/integration` into builder context.
2. Run the packaging contract test and capture failing output (RED).
3. Update `Dockerfile` builder-stage copy set to include `tests/integration`.
4. Re-run packaging contract test (GREEN).
5. Re-run Docker smoke build script to confirm end-to-end conformance.

## Affected Modules
- `Dockerfile`
- `scripts/dev/test-docker-image-packaging.sh`
- `specs/3438/*`

## Risks / Mitigations
- Risk: image context size increases slightly.
  - Mitigation: copy only `tests/integration` directory required by workspace membership.
- Risk: external registry/network instability can fail smoke independently.
  - Mitigation: distinguish infra/network failures from manifest-resolution failures in evidence.

## Interfaces / Contracts
- Docker builder contract: workspace compile for `tau-coding-agent` must succeed.
- Packaging regression contract: missing workspace-member copy fails fast in `scripts/dev/test-docker-image-packaging.sh`.
