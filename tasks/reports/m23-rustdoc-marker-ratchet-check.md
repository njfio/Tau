# M23 Rustdoc Marker Ratchet Check

Generated at: 2026-02-15T18:58:13Z

## Summary

- Floor markers: `3249`
- Baseline total markers: `3249`
- Current total markers: `3249`
- Delta markers: `+0`
- Below floor by: `0`
- Ratchet status: `PASS`

## Negative Crate Deltas

- None

## Reproduction Command

```bash
scripts/dev/rustdoc-marker-ratchet-check.sh \
  --repo-root . \
  --policy-file tasks/policies/m23-doc-ratchet-policy.json \
  --current-json tasks/reports/m23-rustdoc-marker-count.json \
  --output-json tasks/reports/m23-rustdoc-marker-ratchet-check.json \
  --output-md tasks/reports/m23-rustdoc-marker-ratchet-check.md
```
