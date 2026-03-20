set shell := ["bash", "-lc"]

TAU_ENV := "unset OPENAI_API_KEY TAU_API_KEY && export TAU_OPENAI_AUTH_MODE=oauth-token && export TAU_PROVIDER_SUBSCRIPTION_STRICT=true"
SESSION_DIR := ".tau/gateway/openresponses/sessions"
SESSION_FILE := ".tau/gateway/openresponses/sessions/default.jsonl"

session-reset:
	@echo "resetting local default session"
	@mkdir -p {{SESSION_DIR}}
	@rm -f {{SESSION_FILE}}

stack-up:
	@echo "starting unified runtime with fresh auth"
	{{TAU_ENV}}; codex login
	just stack-up-fast

stack-up-fast:
	@echo "starting unified runtime (reuse credentials)"
	{{TAU_ENV}}; ./scripts/run/tau-unified.sh up --auth-mode localhost-dev --model gpt-5.3-codex

stack-down:
	@echo "stopping unified runtime"
	{{TAU_ENV}}; ./scripts/run/tau-unified.sh down || true

restart-stack: stack-down stack-up-fast
	@echo "stack restarted"

rebuild:
	@echo "building tau runtime and tui"
	cargo build -p tau-coding-agent -p tau-tui

tui:
	@echo "launching tau tui from root path"
	{{TAU_ENV}}; ./scripts/run/tau-unified.sh tui --model gpt-5.3-codex --request-timeout-ms 180000 --agent-request-max-retries 0

stack-up-fresh: session-reset stack-up-fast
	@echo "fresh stack is up"

tui-fresh: session-reset tui
	@echo "fresh tui session launched"
