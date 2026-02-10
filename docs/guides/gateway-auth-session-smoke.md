# Gateway Auth Session Smoke

Run all commands from repository root.

This runbook validates gateway password-session auth end-to-end:
- session issuance via `POST /gateway/auth/session`
- authorized `GET /gateway/status` with issued bearer token
- fail-closed invalid password behavior
- fail-closed expired token behavior

## Command

```bash
./scripts/demo/gateway-auth-session.sh
```

Optional timeout per step:

```bash
./scripts/demo/gateway-auth-session.sh --timeout-seconds 20
```

## Expected markers

The script must print these PASS markers:
- `[demo:gateway-auth-session] PASS wait-for-gateway-startup`
- `[demo:gateway-auth-session] PASS gateway-auth-session-issue-valid-password`
- `[demo:gateway-auth-session] PASS gateway-status-authorized-with-issued-token`
- `[demo:gateway-auth-session] PASS gateway-auth-session-invalid-password-fails-closed`
- `[demo:gateway-auth-session] PASS gateway-status-expired-token-fails-closed`
- `[demo:gateway-auth-session] summary: total=5 passed=5 failed=0`

## Troubleshooting

- Startup timeout:
  - confirm localhost port binding is available and no policy blocks loopback listeners.
  - inspect `.tau/demo-gateway-auth-session/gateway-openresponses.log`.

- Valid password step fails:
  - verify local process launch arguments include `--gateway-openresponses-auth-mode password-session`.
  - verify `--gateway-openresponses-auth-password` is non-empty.

- Authorized status step fails:
  - verify issued token from session endpoint is non-empty.
  - verify auth header is sent as `Authorization: Bearer <token>`.

- Invalid password step does not fail closed:
  - verify wrong password payload is being sent.
  - verify endpoint returns HTTP `401`.

- Expired token step does not fail closed:
  - verify session TTL remains short in smoke mode and check host clock skew.
  - verify expired token requests return HTTP `401`.
