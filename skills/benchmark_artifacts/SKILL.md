---
name: benchmark-artifacts
description: Name and validate Tau autonomy benchmark proof artifacts.
---

When writing benchmark proof artifacts:

1. Use deterministic names that include mission id, benchmark id, run id, and proof type.
2. Prefer paths shaped as `artifacts/bench/<benchmark-id>/<mission-id>/<run-id>/<proof-type>.json`.
3. Keep `latest.json` as an index or pointer only; do not make it the sole proof artifact.
4. Include mission id, benchmark id, run id, proof type, generated timestamp, verification gates, and artifact source in the artifact payload when available.
5. When a benchmark task emits multiple files, keep the same prefix and vary only the proof type suffix.
6. Do not claim benchmark proof unless the named artifact exists and can be read.
