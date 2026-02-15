#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BASELINE_SCRIPT="${SCRIPT_DIR}/pr-throughput-baseline.sh"

assert_contains() {
  local haystack="$1"
  local needle="$2"
  local label="$3"
  if [[ "${haystack}" != *"${needle}"* ]]; then
    echo "assertion failed (${label}): expected output to contain '${needle}'" >&2
    echo "actual output:" >&2
    echo "${haystack}" >&2
    exit 1
  fi
}

assert_equals() {
  local expected="$1"
  local actual="$2"
  local label="$3"
  if [[ "${expected}" != "${actual}" ]]; then
    echo "assertion failed (${label}): expected '${expected}' got '${actual}'" >&2
    exit 1
  fi
}

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

fixture_path="${tmp_dir}/fixture.json"
markdown_path="${tmp_dir}/baseline.md"
json_path="${tmp_dir}/baseline.json"
empty_fixture_path="${tmp_dir}/fixture-empty.json"
empty_markdown_path="${tmp_dir}/baseline-empty.md"
empty_json_path="${tmp_dir}/baseline-empty.json"

cat >"${fixture_path}" <<'EOF'
{
  "repository": "fixture/repo",
  "pull_requests": [
    {
      "number": 1,
      "title": "first",
      "url": "https://example.test/pr/1",
      "createdAt": "2026-02-15T00:00:00Z",
      "mergedAt": "2026-02-15T01:00:00Z",
      "reviews": [
        { "submittedAt": "2026-02-15T00:30:00Z" }
      ]
    },
    {
      "number": 2,
      "title": "second",
      "url": "https://example.test/pr/2",
      "createdAt": "2026-02-15T02:00:00Z",
      "mergedAt": "2026-02-15T03:30:00Z",
      "reviews": []
    },
    {
      "number": 3,
      "title": "third",
      "url": "https://example.test/pr/3",
      "createdAt": "2026-02-15T04:00:00Z",
      "mergedAt": "2026-02-15T05:00:00Z",
      "reviews": [
        { "submittedAt": "2026-02-15T04:20:00Z" },
        { "submittedAt": "2026-02-15T04:15:00Z" }
      ]
    }
  ]
}
EOF

cat >"${empty_fixture_path}" <<'EOF'
{
  "repository": "fixture/repo",
  "pull_requests": []
}
EOF

# Functional: fixture-based run writes both markdown and JSON reports.
"${BASELINE_SCRIPT}" \
  --quiet \
  --fixture-json "${fixture_path}" \
  --repo fixture/repo \
  --since-days 0 \
  --limit 10 \
  --generated-at "2026-02-15T12:00:00Z" \
  --output-md "${markdown_path}" \
  --output-json "${json_path}"

if [[ ! -f "${markdown_path}" ]]; then
  echo "assertion failed (functional markdown output): missing ${markdown_path}" >&2
  exit 1
fi
if [[ ! -f "${json_path}" ]]; then
  echo "assertion failed (functional json output): missing ${json_path}" >&2
  exit 1
fi

json_content="$(cat "${json_path}")"
markdown_content="$(cat "${markdown_path}")"

assert_equals "1" "$(jq -r '.schema_version' <<<"${json_content}")" "functional schema version"
assert_equals "3" "$(jq -r '.window.merged_pr_count' <<<"${json_content}")" "functional pr count"
assert_equals "4200" "$(jq -r '.metrics.pr_age.avg_seconds' <<<"${json_content}")" "functional pr-age avg seconds"
assert_equals "3600" "$(jq -r '.metrics.pr_age.p50_seconds' <<<"${json_content}")" "functional pr-age p50 seconds"
assert_equals "1350" "$(jq -r '.metrics.review_latency.avg_seconds' <<<"${json_content}")" "functional review-latency avg seconds"
assert_equals "900" "$(jq -r '.metrics.review_latency.p50_seconds' <<<"${json_content}")" "functional review-latency p50 seconds"
assert_equals "7200" "$(jq -r '.metrics.merge_interval.avg_seconds' <<<"${json_content}")" "functional merge-interval avg seconds"
assert_equals "5400" "$(jq -r '.metrics.merge_interval.p50_seconds' <<<"${json_content}")" "functional merge-interval p50 seconds"
assert_contains "${markdown_content}" "| PR age (created -> merged) | 3 | 1.17h | 1.00h | 1.00h | 1.50h |" "functional markdown metrics row"
assert_contains "${markdown_content}" "| #3 | 2026-02-15T04:00:00Z | 2026-02-15T05:00:00Z | 1.00h | 2026-02-15T04:15:00Z | 15.00m |" "functional markdown newest row"

# Regression: empty fixture yields a valid zero-count report with no crash.
"${BASELINE_SCRIPT}" \
  --quiet \
  --fixture-json "${empty_fixture_path}" \
  --repo fixture/repo \
  --since-days 0 \
  --limit 10 \
  --generated-at "2026-02-15T12:00:00Z" \
  --output-md "${empty_markdown_path}" \
  --output-json "${empty_json_path}"

assert_equals "0" "$(jq -r '.window.merged_pr_count' <"${empty_json_path}")" "regression empty pr count"
assert_contains "$(cat "${empty_markdown_path}")" "| _none_ | - | - | - | - | - |" "regression empty markdown sample"

echo "pr-throughput-baseline tests passed"
