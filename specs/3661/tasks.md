# Tasks: Issue #3661 - Recycle stale tau-unified runtime after repo/runtime fingerprint changes

- [x] T1 (RED): extend launcher tests to cover fingerprint persistence, stale
      runtime recycle on `up`, and stale runtime recycle on bootstrapped `tui`.
- [x] T2 (GREEN): add runtime fingerprint bookkeeping and mismatch detection to
      `scripts/run/tau-unified.sh`.
- [x] T3 (GREEN): integrate stale runtime recycle behavior into `up` and
      `bootstrap_runtime_for_tui`.
- [x] T4 (VERIFY): run scoped launcher verification and capture the stale
      runtime reproduction evidence for `#3661`.

## Tier Mapping
- Regression: stale runtime recycle for `up` and bootstrapped `tui`
- Functional: same-fingerprint reuse remains a no-op
- Integration: launcher bookkeeping stays compatible with existing runner mode
