#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

REPO_SLUG=""
FIXTURE_JSON=""
OUTPUT_MD="${REPO_ROOT}/tasks/reports/pr-throughput-baseline.md"
OUTPUT_JSON="${REPO_ROOT}/tasks/reports/pr-throughput-baseline.json"
LIMIT=60
SINCE_DAYS=30
GENERATED_AT="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
QUIET_MODE="false"

usage() {
  cat <<'EOF'
Usage: pr-throughput-baseline.sh [options]

Generate reproducible PR throughput baseline metrics:
- PR age (created -> merged)
- Review latency (created -> first review)
- Merge interval (time between merged PRs)

Options:
  --repo <owner/name>     Repository slug for gh queries (defaults to current repo).
  --fixture-json <path>   Use fixture JSON instead of live gh data.
  --output-md <path>      Markdown report output path.
  --output-json <path>    JSON report output path.
  --limit <n>             Maximum merged PRs to include after filtering (default: 60).
  --since-days <n>        Include only PRs merged in last N days; 0 disables (default: 30).
  --generated-at <iso>    Override generated timestamp (ISO-8601 UTC).
  --quiet                 Suppress informational output.
  --help                  Show this help text.

Fixture JSON format:
{
  "repository": "owner/repo",
  "pull_requests": [
    {
      "number": 1,
      "title": "example",
      "url": "https://github.com/owner/repo/pull/1",
      "createdAt": "2026-02-15T00:00:00Z",
      "mergedAt": "2026-02-15T01:00:00Z",
      "reviews": [{ "submittedAt": "2026-02-15T00:10:00Z" }]
    }
  ]
}
EOF
}

require_cmd() {
  local name="$1"
  if ! command -v "${name}" >/dev/null 2>&1; then
    echo "error: required command '${name}' not found" >&2
    exit 1
  fi
}

log_info() {
  if [[ "${QUIET_MODE}" != "true" ]]; then
    echo "$@"
  fi
}

format_duration() {
  local value="${1:-null}"
  if [[ -z "${value}" || "${value}" == "null" ]]; then
    printf 'n/a'
    return 0
  fi
  awk -v seconds="${value}" '
    BEGIN {
      if (seconds >= 3600) {
        printf "%.2fh", seconds / 3600.0;
      } else if (seconds >= 60) {
        printf "%.2fm", seconds / 60.0;
      } else {
        printf "%.0fs", seconds;
      }
    }
  '
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --repo)
      REPO_SLUG="$2"
      shift 2
      ;;
    --fixture-json)
      FIXTURE_JSON="$2"
      shift 2
      ;;
    --output-md)
      OUTPUT_MD="$2"
      shift 2
      ;;
    --output-json)
      OUTPUT_JSON="$2"
      shift 2
      ;;
    --limit)
      LIMIT="$2"
      shift 2
      ;;
    --since-days)
      SINCE_DAYS="$2"
      shift 2
      ;;
    --generated-at)
      GENERATED_AT="$2"
      shift 2
      ;;
    --quiet)
      QUIET_MODE="true"
      shift
      ;;
    --help)
      usage
      exit 0
      ;;
    *)
      echo "error: unknown option '$1'" >&2
      usage >&2
      exit 1
      ;;
  esac
done

require_cmd jq

if ! [[ "${LIMIT}" =~ ^[0-9]+$ ]]; then
  echo "error: --limit must be a non-negative integer" >&2
  exit 1
fi
if ! [[ "${SINCE_DAYS}" =~ ^[0-9]+$ ]]; then
  echo "error: --since-days must be a non-negative integer" >&2
  exit 1
fi

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT
raw_input_path="${tmp_dir}/raw-input.json"

if [[ -n "${FIXTURE_JSON}" ]]; then
  if [[ ! -f "${FIXTURE_JSON}" ]]; then
    echo "error: fixture JSON not found: ${FIXTURE_JSON}" >&2
    exit 1
  fi
  cp "${FIXTURE_JSON}" "${raw_input_path}"
  if [[ -z "${REPO_SLUG}" ]]; then
    REPO_SLUG="$(jq -r '.repository // "fixture/repository"' "${raw_input_path}")"
  fi
else
  require_cmd gh
  if [[ -z "${REPO_SLUG}" ]]; then
    REPO_SLUG="$(gh repo view --json nameWithOwner --jq '.nameWithOwner')"
  fi
  # Fetch extra merged PRs before date-window filtering so recent-window selection is stable.
  fetch_limit=200
  if (( LIMIT > fetch_limit )); then
    fetch_limit="${LIMIT}"
  fi
  gh pr list \
    --repo "${REPO_SLUG}" \
    --state merged \
    --limit "${fetch_limit}" \
    --json number,title,url,createdAt,mergedAt,reviews >"${raw_input_path}"
fi

analysis_json="$(
  jq -c \
    --arg generated_at "${GENERATED_AT}" \
    --arg repository "${REPO_SLUG}" \
    --argjson limit "${LIMIT}" \
    --argjson since_days "${SINCE_DAYS}" '
      def stats($arr):
        if ($arr | length) == 0 then
          {
            count: 0,
            avg_seconds: null,
            p50_seconds: null,
            min_seconds: null,
            max_seconds: null
          }
        else
          ($arr | sort) as $sorted |
          {
            count: ($sorted | length),
            avg_seconds: (($sorted | add) / ($sorted | length)),
            p50_seconds: $sorted[(((($sorted | length) - 1) / 2) | floor)],
            min_seconds: $sorted[0],
            max_seconds: $sorted[-1]
          }
        end;

      (if (type == "object" and has("pull_requests")) then .pull_requests else . end) as $source_prs |
      ($source_prs | map(
        select(.createdAt != null and .mergedAt != null) |
        {
          number: .number,
          title: (.title // ""),
          url: (.url // ""),
          created_at: .createdAt,
          merged_at: .mergedAt,
          first_review_at: (
            [ (.reviews // [])[] | select(.submittedAt != null) | .submittedAt ] |
            sort |
            .[0]
          )
        } |
        .pr_age_seconds = ((.merged_at | fromdateiso8601) - (.created_at | fromdateiso8601)) |
        .review_latency_seconds = (
          if .first_review_at == null then
            null
          else
            ((.first_review_at | fromdateiso8601) - (.created_at | fromdateiso8601))
          end
        )
      )) as $normalized |
      (if $since_days > 0 then (now - ($since_days * 86400)) else 0 end) as $cutoff |
      ($normalized
        | map(select(($since_days == 0) or ((.merged_at | fromdateiso8601) >= $cutoff)))
        | sort_by(.merged_at)
      ) as $filtered |
      (if $limit > 0 and ($filtered | length) > $limit then $filtered[-$limit:] else $filtered end) as $rows |
      ($rows | map(.pr_age_seconds)) as $pr_age_values |
      ($rows | map(.review_latency_seconds) | map(select(. != null))) as $review_latency_values |
      (
        [
          range(1; ($rows | length)) as $index |
          (
            (($rows[$index].merged_at | fromdateiso8601) - ($rows[$index - 1].merged_at | fromdateiso8601))
          )
        ]
      ) as $merge_interval_values |
      {
        schema_version: 1,
        generated_at: $generated_at,
        repository: $repository,
        window: {
          limit: $limit,
          since_days: $since_days,
          source_pr_count: ($source_prs | length),
          merged_pr_count: ($rows | length)
        },
        metrics: {
          pr_age: stats($pr_age_values),
          review_latency: stats($review_latency_values),
          merge_interval: stats($merge_interval_values)
        },
        pull_requests: $rows
      }
    ' "${raw_input_path}"
)"

mkdir -p "$(dirname "${OUTPUT_MD}")" "$(dirname "${OUTPUT_JSON}")"
printf '%s\n' "${analysis_json}" | jq '.' >"${OUTPUT_JSON}"

pr_count="$(jq -r '.window.merged_pr_count' <<<"${analysis_json}")"

pr_age_count="$(jq -r '.metrics.pr_age.count' <<<"${analysis_json}")"
pr_age_avg="$(jq -r '.metrics.pr_age.avg_seconds' <<<"${analysis_json}")"
pr_age_p50="$(jq -r '.metrics.pr_age.p50_seconds' <<<"${analysis_json}")"
pr_age_min="$(jq -r '.metrics.pr_age.min_seconds' <<<"${analysis_json}")"
pr_age_max="$(jq -r '.metrics.pr_age.max_seconds' <<<"${analysis_json}")"

review_count="$(jq -r '.metrics.review_latency.count' <<<"${analysis_json}")"
review_avg="$(jq -r '.metrics.review_latency.avg_seconds' <<<"${analysis_json}")"
review_p50="$(jq -r '.metrics.review_latency.p50_seconds' <<<"${analysis_json}")"
review_min="$(jq -r '.metrics.review_latency.min_seconds' <<<"${analysis_json}")"
review_max="$(jq -r '.metrics.review_latency.max_seconds' <<<"${analysis_json}")"

merge_count="$(jq -r '.metrics.merge_interval.count' <<<"${analysis_json}")"
merge_avg="$(jq -r '.metrics.merge_interval.avg_seconds' <<<"${analysis_json}")"
merge_p50="$(jq -r '.metrics.merge_interval.p50_seconds' <<<"${analysis_json}")"
merge_min="$(jq -r '.metrics.merge_interval.min_seconds' <<<"${analysis_json}")"
merge_max="$(jq -r '.metrics.merge_interval.max_seconds' <<<"${analysis_json}")"

{
  cat <<EOF
# PR Throughput Baseline

- Generated at: ${GENERATED_AT}
- Repository: ${REPO_SLUG}
- Sampled merged PRs: ${pr_count}
- Window: last ${SINCE_DAYS} day(s), capped to ${LIMIT} merged PRs
- Reproduce:
  - \`scripts/dev/pr-throughput-baseline.sh --repo ${REPO_SLUG} --since-days ${SINCE_DAYS} --limit ${LIMIT} --output-md ${OUTPUT_MD} --output-json ${OUTPUT_JSON}\`

## Metrics

| Metric | Count | Avg | P50 | Min | Max |
| --- | ---: | ---: | ---: | ---: | ---: |
| PR age (created -> merged) | ${pr_age_count} | $(format_duration "${pr_age_avg}") | $(format_duration "${pr_age_p50}") | $(format_duration "${pr_age_min}") | $(format_duration "${pr_age_max}") |
| Review latency (created -> first review) | ${review_count} | $(format_duration "${review_avg}") | $(format_duration "${review_p50}") | $(format_duration "${review_min}") | $(format_duration "${review_max}") |
| Merge interval (between merged PRs) | ${merge_count} | $(format_duration "${merge_avg}") | $(format_duration "${merge_p50}") | $(format_duration "${merge_min}") | $(format_duration "${merge_max}") |

## Sample (newest merged first)

| PR | Created | Merged | PR age | First review | Review latency |
| --- | --- | --- | ---: | --- | ---: |
EOF

  if (( pr_count == 0 )); then
    echo "| _none_ | - | - | - | - | - |"
  else
    while IFS=$'\t' read -r number created_at merged_at pr_age_seconds first_review_at review_latency_seconds; do
      pr_age_render="$(format_duration "${pr_age_seconds}")"
      review_latency_render="$(format_duration "${review_latency_seconds}")"
      first_review_render="${first_review_at}"
      if [[ -z "${first_review_render}" || "${first_review_render}" == "null" ]]; then
        first_review_render="n/a"
      fi
      echo "| #${number} | ${created_at} | ${merged_at} | ${pr_age_render} | ${first_review_render} | ${review_latency_render} |"
    done < <(
      jq -r '.pull_requests | reverse[] | [.number, .created_at, .merged_at, .pr_age_seconds, (.first_review_at // "n/a"), (.review_latency_seconds // "null")] | @tsv' \
        <<<"${analysis_json}"
    )
  fi
} >"${OUTPUT_MD}"

log_info "wrote throughput baseline report:"
log_info "  - ${OUTPUT_MD}"
log_info "  - ${OUTPUT_JSON}"
