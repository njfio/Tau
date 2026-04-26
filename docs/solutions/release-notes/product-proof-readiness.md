# Product Proof Readiness

## Highlights

Tau now has a canonical executable local product-proof path for reviewers and release handoffs. The proof can run as a fast non-runtime check, execute the local runtime path, emit JSON evidence, and optionally verify the browser-facing webchat surface.

## What's New

- `scripts/dev/prove-tau-product.sh --check` validates the canonical guide, launcher contract, and deterministic live-run contract without starting the real runtime.
- `scripts/dev/prove-tau-product.sh --run` starts Tau through `scripts/run/tau-unified.sh`, checks launcher status, validates `/gateway/status`, runs the TUI live-shell path, and shuts down.
- `--report <path>` writes JSON evidence for check and run modes so reviewers can parse proof status instead of scraping terminal output.
- `--webchat-smoke` is an opt-in live check for `/webchat`; when used with `--run`, it validates stable product-surface markers and records `webchat_url` plus a `webchat` completed step.

## Upgrade Notes

- Existing launcher commands and the default product-proof `--run` behavior remain unchanged.
- Use `--webchat-smoke` only when the local runtime should prove the webchat surface in addition to gateway status and TUI live-shell.
- Consumers of run reports should treat `completed_steps` as the proof sequence. Default runs report `up`, `status`, `gateway_status`, `tui`, `down`; webchat-smoke runs insert `webchat` before `tui`.

## Breaking Changes

None. The proof script options are additive.

## Fixed

- Product-proof failure modes now clean up the fake/live runtime path reliably when gateway status, curl, launcher status, or webchat readiness checks fail.
- JSON report consumers have documented parser examples in `docs/guides/canonical-product-proof.md`.
