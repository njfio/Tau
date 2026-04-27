# Canonical Product Proof

Run commands from the repository root.

## Purpose

This guide defines the shortest repeatable command path that proves Tau can start the unified gateway/dashboard runtime, expose operator endpoints, report runtime status, provide TUI access to runtime artifacts, and shut down cleanly.

Use this when a reviewer asks whether the actual program path is runnable. Use deeper verification gates such as `scripts/verify/m296-ga-readiness-gate.sh` when promoting a release or validating broader operator readiness.

## Prerequisites

Required tools:

- `cargo`
- `curl`
- `jq`

Local smoke posture:

```bash
export TAU_UNIFIED_AUTH_MODE=localhost-dev
export TAU_UNIFIED_BIND=127.0.0.1:8791
```

Provider-backed model runs may also require provider credentials such as `OPENAI_API_KEY`, depending on the selected model. The command path below uses the launcher defaults and should be treated as local operator proof, not a production deployment recipe.

## Executable Smoke

Run the non-destructive proof check before review or CI-style validation:

```bash
./scripts/dev/prove-tau-product.sh --check
```

The check validates this guide, runs the launcher contract test, and contract-tests the live --run path with fake runner/curl hooks without starting the real runtime.

To also emit machine-readable evidence for review or CI logs, pass a report path:

```bash
./scripts/dev/prove-tau-product.sh --check --report /tmp/tau-product-proof-check.json
```

Run the live local proof when you want the script to execute the command path below and clean up the runtime afterward:

```bash
./scripts/dev/prove-tau-product.sh --run
```

The live proof supports the same report option:

```bash
./scripts/dev/prove-tau-product.sh --run --report /tmp/tau-product-proof-run.json
```

To opt into a stronger local product-surface check, add `--webchat-smoke`. This keeps the default live proof short while allowing reviewers to verify that `/webchat` returns stable webchat/dashboard markers:

```bash
./scripts/dev/prove-tau-product.sh --run --webchat-smoke --report /tmp/tau-product-proof-webchat.json
```

To also verify the Gateway sessions API readiness surface, add `--sessions-smoke`. This fetches `/gateway/sessions`, validates the JSON response shape, and records the sessions endpoint in the report:

```bash
./scripts/dev/prove-tau-product.sh --run --sessions-smoke --report /tmp/tau-product-proof-sessions.json
```

To verify the read-only Gateway memory API readiness surface, add `--memory-smoke`. This fetches `/gateway/memory/default`, validates the JSON response shape, and records the memory endpoint in the report:

```bash
./scripts/dev/prove-tau-product.sh --run --memory-smoke --report /tmp/tau-product-proof-memory.json
```

To verify the Gateway channel lifecycle status surface, add `--channel-lifecycle-smoke`. This posts a status action to `/gateway/channels/discord/lifecycle`, validates the JSON response shape, and records the channel lifecycle endpoint in the report:

```bash
./scripts/dev/prove-tau-product.sh --run --channel-lifecycle-smoke --report /tmp/tau-product-proof-channel-lifecycle.json
```

## Consuming Report JSON

Use report output when a reviewer, CI job, or release handoff needs machine-readable proof instead of terminal text. A successful `--check` report contains `mode`, `status`, and a `checks` object for `guide_contract`, `launcher_contract`, and `run_contract`.

```bash
report=$(mktemp)
./scripts/dev/prove-tau-product.sh --check --report "$report"
python3 - "$report" <<'PY'
import json
import sys

with open(sys.argv[1], encoding="utf-8") as handle:
	payload = json.load(handle)

assert payload["mode"] == "check"
assert payload["status"] == "passed"
assert payload["checks"]["guide_contract"] == "passed"
assert payload["checks"]["launcher_contract"] == "passed"
assert payload["checks"]["run_contract"] == "passed"
PY
```

A successful `--run` report adds the runtime evidence fields `bind`, `auth_mode`, `model`, `gateway_status_url`, and `completed_steps`. Consumers should treat the step list as the proof sequence and assert that the live path reached shutdown. When `--webchat-smoke` is used, the report also includes `webchat_url` and inserts `webchat` between `gateway_status` and `tui` in `completed_steps`. When `--sessions-smoke` is used, the report includes `gateway_sessions_url` and inserts `sessions_api` before `tui`. When `--memory-smoke` is used, the report includes `gateway_memory_url` and inserts `memory_api` before `tui`. When `--channel-lifecycle-smoke` is used, the report includes `gateway_channel_lifecycle_url` and inserts `channel_lifecycle_api` before `tui`.

```bash
report=/tmp/tau-product-proof-run.json
./scripts/dev/prove-tau-product.sh --run --report "$report"
python3 - "$report" <<'PY'
import json
import sys

with open(sys.argv[1], encoding="utf-8") as handle:
	payload = json.load(handle)

assert payload["mode"] == "run"
assert payload["status"] == "passed"
assert payload["gateway_status_url"].endswith("/gateway/status")
assert payload["completed_steps"] == ["up", "status", "gateway_status", "tui", "down"]
PY
```

For the opt-in webchat proof, assert the additional URL and step:

```bash
report=/tmp/tau-product-proof-webchat.json
./scripts/dev/prove-tau-product.sh --run --webchat-smoke --report "$report"
python3 - "$report" <<'PY'
import json
import sys

with open(sys.argv[1], encoding="utf-8") as handle:
	payload = json.load(handle)

assert payload["mode"] == "run"
assert payload["status"] == "passed"
assert payload["webchat_url"].endswith("/webchat")
assert payload["completed_steps"] == ["up", "status", "gateway_status", "webchat", "tui", "down"]
PY
```

For the opt-in sessions proof, assert the Gateway sessions URL and step:

```bash
report=/tmp/tau-product-proof-sessions.json
./scripts/dev/prove-tau-product.sh --run --sessions-smoke --report "$report"
python3 - "$report" <<'PY'
import json
import sys

with open(sys.argv[1], encoding="utf-8") as handle:
	payload = json.load(handle)

assert payload["mode"] == "run"
assert payload["status"] == "passed"
assert payload["gateway_sessions_url"].endswith("/gateway/sessions")
assert payload["completed_steps"] == ["up", "status", "gateway_status", "sessions_api", "tui", "down"]
PY
```

For the opt-in memory proof, assert the Gateway memory URL and step:

```bash
report=/tmp/tau-product-proof-memory.json
./scripts/dev/prove-tau-product.sh --run --memory-smoke --report "$report"
python3 - "$report" <<'PY'
import json
import sys

with open(sys.argv[1], encoding="utf-8") as handle:
	payload = json.load(handle)

assert payload["mode"] == "run"
assert payload["status"] == "passed"
assert payload["gateway_memory_url"].endswith("/gateway/memory/default")
assert payload["completed_steps"] == ["up", "status", "gateway_status", "memory_api", "tui", "down"]
PY
```

For the opt-in channel lifecycle proof, assert the Gateway channel lifecycle URL and step:

```bash
report=/tmp/tau-product-proof-channel-lifecycle.json
./scripts/dev/prove-tau-product.sh --run --channel-lifecycle-smoke --report "$report"
python3 - "$report" <<'PY'
import json
import sys

with open(sys.argv[1], encoding="utf-8") as handle:
	payload = json.load(handle)

assert payload["mode"] == "run"
assert payload["status"] == "passed"
assert payload["gateway_channel_lifecycle_url"].endswith("/gateway/channels/discord/lifecycle")
assert payload["completed_steps"] == ["up", "status", "gateway_status", "channel_lifecycle_api", "tui", "down"]
PY
```

If the script exits nonzero, do not parse a partial report as success. Use the terminal error and `.tau/unified/tau-unified.log` for triage, then regenerate the report after the proof passes.

## Command Path

1. Start the unified runtime.

```bash
./scripts/run/tau-unified.sh up --auth-mode localhost-dev
```

2. Confirm the launcher sees the runtime process and artifact paths.

```bash
./scripts/run/tau-unified.sh status
```

3. Check gateway status through the exposed endpoint.

```bash
curl -sS http://127.0.0.1:8791/gateway/status | jq
```

4. Optional: check the webchat product surface.

```bash
curl -sS http://127.0.0.1:8791/webchat | grep -F "Tau Gateway Webchat"
```

5. Optional: check the Gateway sessions API readiness surface.

```bash
curl -sS http://127.0.0.1:8791/gateway/sessions | jq '.sessions'
```

6. Optional: check the read-only Gateway memory API readiness surface.

```bash
curl -sS http://127.0.0.1:8791/gateway/memory/default | jq '.exists'
```

7. Optional: check the Gateway channel lifecycle status readiness surface.

```bash
curl -sS -X POST -H 'Content-Type: application/json' -d '{"action":"status"}' http://127.0.0.1:8791/gateway/channels/discord/lifecycle | jq '.report'
```

8. Check the read-only dashboard artifact path through live-shell mode.

```bash
./scripts/run/tau-unified.sh tui --live-shell --iterations 1 --interval-ms 1000 --no-color
```

9. Stop the runtime.

```bash
./scripts/run/tau-unified.sh down
```

## Expected Evidence

The `up` command prints:

- `tau-unified: started`
- `tau-unified: webchat=http://127.0.0.1:8791/webchat`
- `tau-unified: ops=http://127.0.0.1:8791/ops`
- `tau-unified: dashboard=http://127.0.0.1:8791/dashboard`
- `tau-unified: log=<repo>/.tau/unified/tau-unified.log`

The `status` command prints:

- `tau-unified: running pid=<pid>`
- `tau-unified: pid_file=<repo>/.tau/unified/tau-unified.pid`
- `tau-unified: log_file=<repo>/.tau/unified/tau-unified.log`
- `tau-unified: command_file=<repo>/.tau/unified/tau-unified.last-cmd`
- `tau-unified: fingerprint_file=<repo>/.tau/unified/tau-unified.runtime-fingerprint`

The gateway status endpoint returns JSON. In a sparse local state, `gateway/status` should report a healthy gateway posture while other subsystems may still be in an expected local hold posture until live activity produces their artifacts.

The live-shell command should print dashboard/runtime artifact status without requiring an interactive TTY. The `down` command should print `tau-unified: stopped` and remove the runtime pid/fingerprint files.

## Failure Triage

If `up` fails, inspect `.tau/unified/tau-unified.log` first. If the bind address is occupied, change the port:

```bash
TAU_UNIFIED_BIND=127.0.0.1:8792 ./scripts/run/tau-unified.sh up --auth-mode localhost-dev
```

If `status` says the runtime is not running, re-run `up` and inspect `.tau/unified/tau-unified.last-cmd` to confirm the launcher command matches the expected auth mode and bind address.

If the gateway status endpoint fails in token or password-session mode, switch back to `localhost-dev` for this proof path, then use `docs/guides/operator-deployment-guide.md` for authenticated deployment validation.

## Related Gates

- `scripts/dev/prove-tau-product.sh --check` verifies this proof surface without starting the real runtime.
- `scripts/dev/prove-tau-product.sh --run` executes the live proof path and runs `down` during cleanup.
- `scripts/run/test-tau-unified.sh` verifies the launcher contract without starting the real runtime.
- `scripts/dev/operator-readiness-live-check.sh` verifies gateway and operator readiness against a running local endpoint.
- `scripts/verify/m296-ga-readiness-gate.sh` aggregates the larger connected operator GA readiness flow.