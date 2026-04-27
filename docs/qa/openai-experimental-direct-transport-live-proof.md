# OpenAI Experimental Direct Transport Live Proof

The `openai_experimental_direct` proof is opt-in because direct OpenAI OAuth/session bearer use is not a documented public API contract. Normal CI runs the selector and skip-safe live probe without credentials:

```bash
cargo test -p tau-provider openai_experimental_direct_transport -- --test-threads=1
```

To run the live probe, provide a bearer token through the environment. The script never prints the token and falls back to the skip-safe test path when the opt-in or token is missing:

```bash
TAU_OPENAI_EXPERIMENTAL_DIRECT_LIVE=1 \
TAU_OPENAI_EXPERIMENTAL_DIRECT_BEARER="$TOKEN" \
scripts/verify/openai-experimental-direct-transport-live.sh
```

Optional overrides:

```bash
TAU_OPENAI_EXPERIMENTAL_DIRECT_API_BASE=https://api.openai.com/v1
TAU_OPENAI_EXPERIMENTAL_DIRECT_MODEL=gpt-5.2-codex
```

The probe validates Tau's provider selection predicate for session bearer credentials, then sends a minimal direct Responses request with `Authorization: Bearer ...` to `/v1/responses`. Official OpenAI documentation currently describes `POST /v1/responses`, `input`, `model`, `max_output_tokens`, `tool_choice`, `tools`, and bearer authentication for API-key usage; session/OAuth direct bearer usage remains experimental and reversible.