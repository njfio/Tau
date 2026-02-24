# Tasks: Issue #3438 - Docker workspace-member packaging fix

1. [x] T1 (RED, Regression): add failing packaging contract assertion for required `tests/integration` Dockerfile copy.
2. [x] T2 (GREEN, Functional): update Dockerfile builder context to include `tests/integration`.
3. [x] T3 (GREEN, Regression): run `scripts/dev/test-docker-image-packaging.sh` and confirm pass.
4. [x] T4 (VERIFY, Conformance): run `scripts/dev/docker-image-smoke.sh --tag tau-coding-agent:local-smoke`.
5. [x] T5 (VERIFY): run scoped hygiene (`cargo fmt --all -- --check`) and set spec status to `Implemented`.

## RED / GREEN Evidence
### RED
- `bash scripts/dev/test-docker-image-packaging.sh`
- observed failure: `error: Dockerfile must copy tests/integration for workspace member resolution`.
- `bash scripts/dev/docker-image-smoke.sh --tag tau-coding-agent:local-smoke`
- observed failure before base-image adjustment: `ort_sys` linker errors on Bookworm (`__isoc23_*`, `__cxa_call_terminate`).

### GREEN
- `bash scripts/dev/test-docker-image-packaging.sh`
- `bash scripts/dev/docker-image-smoke.sh --tag tau-coding-agent:local-smoke`
- `cargo fmt --all -- --check`
