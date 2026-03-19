set shell := ["bash", "-lc"]

REPO_ROOT := justfile_directory()
TAU_ENV := "unset OPENAI_API_KEY TAU_API_KEY && export TAU_OPENAI_AUTH_MODE=oauth-token && export TAU_PROVIDER_SUBSCRIPTION_STRICT=true"
GATEWAY_PORT := "8791"
RUNTIME_DIR := ".tau/unified"
GATEWAY_STATE_DIR := ".tau/gateway"
DASHBOARD_STATE_DIR := ".tau/dashboard"
RUNTIME_LOG := ".tau/unified/tau-unified.log"
RUNTIME_PID := ".tau/unified/tau-unified.pid"
RUNTIME_CMD_FILE := ".tau/unified/tau-unified.last-cmd"
OPENRESPONSES_SESSION_DIR := ".tau/gateway/openresponses/sessions"
DEFAULT_OPENRESPONSES_SESSION := ".tau/gateway/openresponses/sessions/default.jsonl"
RUNTIME_SESSION := "tau-runtime"
TUI_SESSION := "tau-tui"
RUNTIME_CMD := "target/debug/tau-coding-agent --model gpt-5.3-codex --gateway-state-dir .tau/gateway --dashboard-state-dir .tau/dashboard --gateway-openresponses-server --gateway-openresponses-bind 127.0.0.1:8791 --gateway-openresponses-auth-mode localhost-dev --gateway-openresponses-max-input-chars 32000 --request-timeout-ms 180000 --agent-request-max-retries 0 --provider-max-retries 0"

[private]
clear-stale-gateway-listener:
	@pids=$(lsof -tiTCP:{{GATEWAY_PORT}} -sTCP:LISTEN 2>/dev/null || true); \
	if [ -z "$pids" ]; then \
	  exit 0; \
	fi; \
	for pid in $pids; do \
	  cmd=$(ps -p $pid -o command=); \
	  case "$cmd" in \
	    *tau-coding-agent*--gateway-openresponses-server*) \
	      echo "stopping stale tau gateway listener $pid"; \
	      kill $pid || true ;; \
	    *) \
	      echo "refusing to kill non-tau listener on {{GATEWAY_PORT}}: $pid $cmd"; \
	      exit 1 ;; \
	  esac; \
	done; \
	sleep 1; \
	remaining=$(lsof -tiTCP:{{GATEWAY_PORT}} -sTCP:LISTEN 2>/dev/null || true); \
	if [ -n "$remaining" ]; then \
	  echo "listener still active on {{GATEWAY_PORT}}: $remaining"; \
	  exit 1; \
	fi

[private]
stop-tmux-runtime:
	@tmux kill-session -t {{RUNTIME_SESSION}} >/dev/null 2>&1 || true
	@tmux kill-session -t {{TUI_SESSION}} >/dev/null 2>&1 || true

[private]
verify-runtime:
	@pid=$(cat {{RUNTIME_PID}}); \
	if ! kill -0 $pid >/dev/null 2>&1; then \
	  echo "tau runtime exited before verification"; \
	  tail -n 80 {{RUNTIME_LOG}} || true; \
	  exit 1; \
	fi; \
	if ! lsof -nP -iTCP:{{GATEWAY_PORT}} -sTCP:LISTEN >/dev/null 2>&1; then \
	  echo "tau runtime did not bind {{GATEWAY_PORT}}"; \
	  tail -n 80 {{RUNTIME_LOG}} || true; \
	  exit 1; \
	fi

session-reset:
	@echo "resetting local default session"
	@mkdir -p {{OPENRESPONSES_SESSION_DIR}}
	@rm -f {{DEFAULT_OPENRESPONSES_SESSION}}

stack-up:
	@echo "starting dashboard/gateway runtime with fresh auth"
	{{TAU_ENV}}; codex login
	just stack-up-fast

stack-up-fresh:
	@echo "starting dashboard/gateway runtime with fresh local session"
	just session-reset
	just stack-up-fast

stack-up-fast:
	@echo "starting dashboard/gateway runtime (reuse credentials)"
	just stack-down
	{{TAU_ENV}}; cargo build -q -p tau-coding-agent
	{{TAU_ENV}}; mkdir -p {{RUNTIME_DIR}} {{GATEWAY_STATE_DIR}} {{DASHBOARD_STATE_DIR}}
	{{TAU_ENV}}; printf '%s\n' '{{RUNTIME_CMD}}' > {{RUNTIME_CMD_FILE}}
	{{TAU_ENV}}; : > {{RUNTIME_LOG}}
	@tmux new-session -d -s {{RUNTIME_SESSION}} "cd {{REPO_ROOT}} && {{TAU_ENV}} && exec {{RUNTIME_CMD}}"
	@tmux pipe-pane -o -t {{RUNTIME_SESSION}} "cat >> {{REPO_ROOT}}/{{RUNTIME_LOG}}"
	@tmux list-panes -t {{RUNTIME_SESSION}} -F '#{pane_pid}' > {{RUNTIME_PID}}
	@sleep 3
	@just verify-runtime
	@echo "tau-unified: started (pid=$(cat {{RUNTIME_PID}})) profile=local-dev"
	@echo "tau-unified: webchat=http://127.0.0.1:8791/webchat"
	@echo "tau-unified: ops=http://127.0.0.1:8791/ops"
	@echo "tau-unified: dashboard=http://127.0.0.1:8791/dashboard"
	@echo "tau-unified: log={{REPO_ROOT}}/{{RUNTIME_LOG}}"

stack-down:
	@echo "stopping the unified stack"
	{{TAU_ENV}}; ./scripts/run/tau-unified.sh down || true
	@just stop-tmux-runtime
	@if [ -f {{RUNTIME_PID}} ]; then \
	  pid=$(cat {{RUNTIME_PID}}); \
	  if kill -0 $pid >/dev/null 2>&1; then \
	    echo "stopping tracked tau runtime $pid"; \
	    kill $pid || true; \
	  fi; \
	  rm -f {{RUNTIME_PID}}; \
	fi
	just clear-stale-gateway-listener

tui:
	@echo "running tau-tui interactive"
	{{TAU_ENV}}; cargo run -q -p tau-tui -- interactive --profile ops-interactive

tui-fresh:
	@echo "running tau-tui interactive with fresh local session"
	just session-reset
	just tui

tui-tmux:
	@echo "launching tau-tui in tmux session {{TUI_SESSION}}"
	@tmux kill-session -t {{TUI_SESSION}} >/dev/null 2>&1 || true
	@tmux new-session -d -s {{TUI_SESSION}} "cd {{REPO_ROOT}} && {{TAU_ENV}} && cargo run -q -p tau-tui -- interactive --profile ops-interactive"
	@echo "attach with: tmux attach -t {{TUI_SESSION}}"

restart-stack: stack-down stack-up
	@echo "stack restarted and ready"

rebuild:
	@echo "clean rebuild of tau binaries"
	cargo clean
	cargo build -p tau-coding-agent -p tau-tui

cycle:
	@echo "full cycle: rebuild, stack, run tui, stop"
	just rebuild
	just stack-up
	just tui
	just stack-down
