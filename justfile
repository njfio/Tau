set shell := ["bash", "-lc"]

TAU_ENV := "unset OPENAI_API_KEY TAU_API_KEY && export TAU_OPENAI_AUTH_MODE=oauth-token && export TAU_PROVIDER_SUBSCRIPTION_STRICT=true"

stack-up:
	@echo "starting dashboard/gateway/TUI runtime"
	$(TAU_ENV) codex login
	$(TAU_ENV) ./scripts/run/tau-unified.sh down || true
	$(TAU_ENV) ./scripts/run/tau-unified.sh up --auth-mode localhost-dev

tui:
	@echo "running tau-tui interactive"
	$(TAU_ENV) ./scripts/run/tau-unified.sh tui --request-timeout-ms 180000 --agent-request-max-retries 0

stack-down:
	@echo "stopping the unified stack"
	$(TAU_ENV) ./scripts/run/tau-unified.sh down || true

restart-stack: stack-down stack-up
	@echo "stack restarted and ready"

rebuild:
	@echo "clean rebuild of tau-tui"
	cargo clean
	cargo build -p tau-tui

cycle:
	@echo "full cycle: rebuild, stack, run tui, stop"
	just rebuild
	just stack-up
	just tui
	just stack-down
