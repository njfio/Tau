#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="${REPO_ROOT}/scripts/dev/rustdoc-marker-threshold-verify.sh"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

cat > "${tmp_dir}/baseline.json" <<'JSON'
{
  "schema_version": 1,
  "total_markers": 100,
  "crates": [
    { "crate": "crate-a", "markers": 30, "files_scanned": 2 },
    { "crate": "crate-b", "markers": 70, "files_scanned": 4 }
  ]
}
JSON

cat > "${tmp_dir}/current.json" <<'JSON'
{
  "schema_version": 1,
  "total_markers": 140,
  "crates": [
    { "crate": "crate-a", "markers": 60, "files_scanned": 3 },
    { "crate": "crate-b", "markers": 65, "files_scanned": 4 },
    { "crate": "crate-c", "markers": 15, "files_scanned": 1 }
  ]
}
JSON

output_json="${tmp_dir}/verify.json"
output_md="${tmp_dir}/verify.md"

"${SCRIPT}" \
  --repo-root "${REPO_ROOT}" \
  --baseline-json "${tmp_dir}/baseline.json" \
  --current-json "${tmp_dir}/current.json" \
  --threshold 150 \
  --generated-at "2026-02-15T00:00:00Z" \
  --output-json "${output_json}" \
  --output-md "${output_md}" \
  --quiet

python3 - "${output_json}" "${output_md}" <<'PY'
import json
import pathlib
import sys

json_path = pathlib.Path(sys.argv[1])
md_path = pathlib.Path(sys.argv[2])

payload = json.loads(json_path.read_text(encoding="utf-8"))
assert payload["schema_version"] == 1
assert payload["baseline_total_markers"] == 100
assert payload["current_total_markers"] == 140
assert payload["delta_total_markers"] == 40
assert payload["remaining_to_threshold"] == 10
assert payload["meets_threshold"] is False

crate_deltas = {row["crate"]: row for row in payload["crate_deltas"]}
assert crate_deltas["crate-a"]["delta_markers"] == 30
assert crate_deltas["crate-b"]["delta_markers"] == -5
assert crate_deltas["crate-c"]["baseline_markers"] == 0
assert crate_deltas["crate-c"]["delta_markers"] == 15

md = md_path.read_text(encoding="utf-8")
assert "Gate status: `FAIL`" in md
assert "| crate-a | 30 | 60 | +30 |" in md
assert "| crate-b | 70 | 65 | -5 |" in md
PY

echo "ok - rustdoc marker threshold verify contract"
