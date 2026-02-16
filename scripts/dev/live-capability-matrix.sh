#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

KEY_FILE="${TAU_PROVIDER_KEYS_FILE:-${REPO_ROOT}/.tau/provider-keys.env}"
OUTPUT_ROOT="${TAU_LIVE_CAPABILITY_OUTPUT_ROOT:-${REPO_ROOT}/.tau/reports/live-validation}"
RUN_ID="${TAU_LIVE_CAPABILITY_RUN_ID:-$(date -u +"%Y%m%d-%H%M%S")}"
RUN_DIR="${OUTPUT_ROOT}/${RUN_ID}-capability-matrix"
TIMEOUT_MS="${TAU_LIVE_CAPABILITY_TIMEOUT_MS:-180000}"
MAX_TURNS="${TAU_LIVE_CAPABILITY_MAX_TURNS:-12}"
CASES_CSV="${TAU_LIVE_CAPABILITY_CASES:-research_openai,blog_anthropic,snake_google,snake_deepseek,blog_xai}"
TAU_BIN="${TAU_LIVE_CAPABILITY_BIN:-${REPO_ROOT}/target/debug/tau-coding-agent}"
SKIP_BUILD="false"

usage() {
  cat <<'EOF'
Usage: live-capability-matrix.sh [options]

Run deterministic live capability scenarios (research/blog/snake) across
configured providers and emit logs + summary under .tau/reports/live-validation.

Options:
  --key-file <path>        Provider key env file (default: .tau/provider-keys.env)
  --output-root <path>     Report root directory (default: .tau/reports/live-validation)
  --run-id <id>            Run identifier prefix (default: UTC timestamp)
  --timeout-ms <ms>        Provider request timeout in milliseconds (default: 180000)
  --max-turns <n>          Max agent turns per case (default: 12)
  --cases <csv>            Comma list of case ids to run
  --bin <path>             tau-coding-agent binary path
  --skip-build             Do not build tau-coding-agent if --bin is missing
  --help                   Show this help text
EOF
}

log() {
  printf '%s\n' "$*"
}

require_cmd() {
  local name="$1"
  if ! command -v "$name" >/dev/null 2>&1; then
    echo "error: required command not found: $name" >&2
    exit 1
  fi
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --key-file)
      KEY_FILE="$2"
      shift 2
      ;;
    --output-root)
      OUTPUT_ROOT="$2"
      shift 2
      ;;
    --run-id)
      RUN_ID="$2"
      shift 2
      ;;
    --timeout-ms)
      TIMEOUT_MS="$2"
      shift 2
      ;;
    --max-turns)
      MAX_TURNS="$2"
      shift 2
      ;;
    --cases)
      CASES_CSV="$2"
      shift 2
      ;;
    --bin)
      TAU_BIN="$2"
      shift 2
      ;;
    --skip-build)
      SKIP_BUILD="true"
      shift
      ;;
    --help|-h)
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

require_cmd grep
require_cmd awk
require_cmd sed

if [[ ! -f "$KEY_FILE" ]]; then
  cat <<MSG
error: provider key file not found: $KEY_FILE
Create it from template:
  cp scripts/dev/provider-keys.env.example .tau/provider-keys.env
  chmod 600 .tau/provider-keys.env
MSG
  exit 1
fi

if ! [[ "$TIMEOUT_MS" =~ ^[0-9]+$ ]]; then
  echo "error: --timeout-ms must be an integer" >&2
  exit 1
fi

if ! [[ "$MAX_TURNS" =~ ^[0-9]+$ ]]; then
  echo "error: --max-turns must be an integer" >&2
  exit 1
fi

set -a
# shellcheck disable=SC1090
source "$KEY_FILE"
set +a

OPENAI_KEY="${OPENAI_API_KEY:-${TAU_API_KEY:-}}"
OPENROUTER_KEY="${OPENROUTER_API_KEY:-${TAU_OPENROUTER_API_KEY:-}}"
ANTHROPIC_KEY="${ANTHROPIC_API_KEY:-}"
GOOGLE_KEY="${GEMINI_API_KEY:-${GOOGLE_API_KEY:-}}"

if [[ ! -x "$TAU_BIN" ]]; then
  if [[ "$SKIP_BUILD" == "true" ]]; then
    echo "error: tau binary is not executable and --skip-build is set: $TAU_BIN" >&2
    exit 1
  fi
  log "[build] compiling tau-coding-agent"
  cargo build -p tau-coding-agent --quiet
fi

if [[ ! -x "$TAU_BIN" ]]; then
  echo "error: tau binary is not executable: $TAU_BIN" >&2
  exit 1
fi

mkdir -p "$RUN_DIR"
SUMMARY_TSV="${RUN_DIR}/summary.tsv"
printf "case\tmodel\trc\tcompletion\ttool_calls\tartifact\tnotes\n" >"$SUMMARY_TSV"

declare -a CASE_IDS=()
IFS=',' read -r -a CASE_IDS <<<"$CASES_CSV"

case_meta() {
  local case_id="$1"
  case "$case_id" in
    research_openai)
      printf "%s\n" "openai/gpt-5.2|openai|research"
      ;;
    blog_anthropic)
      printf "%s\n" "anthropic/claude-opus-4-6|anthropic|blog"
      ;;
    snake_google)
      printf "%s\n" "google/gemini-2.5-pro|google|snake"
      ;;
    snake_deepseek)
      printf "%s\n" "openrouter/deepseek/deepseek-chat-v3.1|openrouter|snake"
      ;;
    blog_xai)
      printf "%s\n" "openrouter/x-ai/grok-4.1-fast|openrouter|blog"
      ;;
    *)
      return 1
      ;;
  esac
}

write_prompt() {
  local task="$1"
  local prompt_file="$2"
  case "$task" in
    research)
      cat >"$prompt_file" <<'EOF'
Research the current state of Rust async runtime tradeoffs.
Use tools to create exactly one file named report.md in the current directory.
report.md must contain:
1) a short executive summary,
2) at least 3 source links,
3) a recommendation section.
Do not just describe steps; actually write report.md.
After writing the file, respond with COMPLETE.
EOF
      ;;
    blog)
      cat >"$prompt_file" <<'EOF'
Build a complete static personal blog in the current directory.
Use tools to create index.html, styles.css, and main.js.
Requirements:
1) index links styles.css and main.js,
2) at least 3 sample posts are rendered,
3) include responsive mobile layout and a simple search/filter interaction.
Do not just explain; write the files.
After writing files, respond with COMPLETE.
EOF
      ;;
    snake)
      cat >"$prompt_file" <<'EOF'
Build a playable browser Snake game in the current directory.
Use tools to create index.html and game.js (styles can be inline or separate).
Requirements:
1) keyboard arrow controls,
2) score display,
3) restart on game over.
Do not just explain; write the files.
After writing files, respond with COMPLETE.
EOF
      ;;
    *)
      echo "error: unknown task '$task'" >&2
      return 1
      ;;
  esac
}

provider_args() {
  local provider="$1"
  PROVIDER_ARGS=()
  case "$provider" in
    openai)
      if [[ -z "$OPENAI_KEY" ]]; then
        return 10
      fi
      PROVIDER_ARGS=(
        --openai-api-key "$OPENAI_KEY"
        --openai-auth-mode api-key
        --api-base "${TAU_OPENAI_API_BASE:-https://api.openai.com/v1}"
      )
      ;;
    anthropic)
      if [[ -z "$ANTHROPIC_KEY" ]]; then
        return 10
      fi
      PROVIDER_ARGS=(
        --anthropic-api-key "$ANTHROPIC_KEY"
        --anthropic-auth-mode api-key
      )
      ;;
    google)
      if [[ -z "$GOOGLE_KEY" ]]; then
        return 10
      fi
      PROVIDER_ARGS=(
        --google-api-key "$GOOGLE_KEY"
        --google-auth-mode api-key
      )
      ;;
    openrouter)
      if [[ -z "$OPENROUTER_KEY" ]]; then
        return 10
      fi
      PROVIDER_ARGS=(
        --openai-api-key "$OPENROUTER_KEY"
        --openai-auth-mode api-key
        --api-base "${TAU_OPENROUTER_API_BASE:-https://openrouter.ai/api/v1}"
      )
      ;;
    *)
      echo "error: unknown provider '$provider'" >&2
      return 1
      ;;
  esac
}

check_research_artifacts() {
  local workspace="$1"
  local report="${workspace}/report.md"
  [[ -f "$report" ]] || return 1
  local link_count
  link_count="$(grep -Eo 'https?://[^ )]+' "$report" | wc -l | tr -d ' ')"
  [[ "${link_count:-0}" -ge 3 ]]
}

check_blog_artifacts() {
  local workspace="$1"
  local index="${workspace}/index.html"
  local css="${workspace}/styles.css"
  local js="${workspace}/main.js"
  [[ -f "$index" && -f "$css" && -f "$js" ]] || return 1
  grep -qi "styles.css" "$index"
  grep -qi "main.js" "$index"
}

check_snake_artifacts() {
  local workspace="$1"
  local index="${workspace}/index.html"
  local js="${workspace}/game.js"
  [[ -f "$index" && -f "$js" ]] || return 1
  grep -qi "canvas" "$index"
  grep -Eqi "keydown|arrow" "$js"
}

run_case() {
  local case_id="$1"
  local meta model provider task
  if ! meta="$(case_meta "$case_id")"; then
    echo "error: unsupported case id '$case_id'" >&2
    return 1
  fi
  model="${meta%%|*}"
  meta="${meta#*|}"
  provider="${meta%%|*}"
  task="${meta#*|}"

  local case_dir="${RUN_DIR}/${case_id}"
  local workspace="${case_dir}/workspace"
  local prompt_file="${case_dir}/prompt.txt"
  local log_file="${case_dir}/run.log"
  mkdir -p "$workspace"
  write_prompt "$task" "$prompt_file"

  local rc completion tool_calls artifact notes
  rc=0
  completion="FAIL"
  tool_calls=0
  artifact="FAIL"
  notes=""

  if ! provider_args "$provider"; then
    rc="SKIP"
    completion="SKIP"
    artifact="SKIP"
    notes="missing_provider_key"
    printf "%s\t%s\t%s\t%s\t%s\t%s\t%s\n" \
      "$case_id" "$model" "$rc" "$completion" "$tool_calls" "$artifact" "$notes" >>"$SUMMARY_TSV"
    return 0
  fi

  log "[run] $case_id ($model)"
  if (
    cd "$workspace"
    TAU_ONBOARD_AUTO=false "$TAU_BIN" \
      --model "$model" \
      --max-turns "$MAX_TURNS" \
      --request-timeout-ms "$TIMEOUT_MS" \
      --provider-subscription-strict=true \
      --json-events \
      --stream-output false \
      --session "${case_dir}/session.sqlite" \
      --prompt-file "$prompt_file" \
      "${PROVIDER_ARGS[@]}"
  ) >"$log_file" 2>&1; then
    rc=0
  else
    rc=$?
  fi

  if grep -q '"type":"agent_end"' "$log_file"; then
    completion="PASS"
  fi

  tool_calls="$(grep -c '"type":"tool_execution_start"' "$log_file" || true)"

  case "$task" in
    research)
      if check_research_artifacts "$workspace"; then artifact="PASS"; fi
      ;;
    blog)
      if check_blog_artifacts "$workspace"; then artifact="PASS"; fi
      ;;
    snake)
      if check_snake_artifacts "$workspace"; then artifact="PASS"; fi
      ;;
  esac

  if [[ "$rc" != "0" ]]; then
    notes="non_zero_exit"
  fi
  if [[ "$completion" != "PASS" ]]; then
    if [[ -n "$notes" ]]; then notes="${notes},"; fi
    notes="${notes}missing_agent_end"
  fi
  if [[ "$artifact" != "PASS" ]]; then
    if [[ -n "$notes" ]]; then notes="${notes},"; fi
    notes="${notes}missing_expected_artifacts"
  fi
  if [[ -z "$notes" ]]; then
    notes="ok"
  fi

  printf "%s\t%s\t%s\t%s\t%s\t%s\t%s\n" \
    "$case_id" "$model" "$rc" "$completion" "$tool_calls" "$artifact" "$notes" >>"$SUMMARY_TSV"
}

for case_id in "${CASE_IDS[@]}"; do
  case_id="$(echo "$case_id" | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//')"
  if [[ -z "$case_id" ]]; then
    continue
  fi
  run_case "$case_id"
done

log ""
log "live capability matrix summary:"
cat "$SUMMARY_TSV"
log ""
log "artifacts: $RUN_DIR"
