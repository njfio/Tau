# ADR 0002: `serenity` TLS backend = `native_tls_backend`

- **Status**: Accepted
- **Date**: 2026-04-23
- **Deciders**: maintainer + Gyre SE agent

## Context

`serenity 0.12.5` (latest published on crates.io) locks its `rustls_backend`
feature to `tokio-rustls 0.25` → `rustls 0.22.4` → `rustls-webpki 0.102.8`.
Between 2026-03-20 and 2026-04-22 four advisories were published against that
`rustls-webpki` version:

- RUSTSEC-2026-0049 — CRL authority matching
- RUSTSEC-2026-0098 — name-constraint URI acceptance
- RUSTSEC-2026-0099 — name-constraint wildcard acceptance
- RUSTSEC-2026-0104 — reachable panic in CRL parsing

We cannot bump the upstream `rustls` version without a new `serenity` release,
and there is no `serenity 0.12.6` on crates.io. The `tau-discord-runtime` crate
is a TLS *client* talking to `discord.com` — a known-good public endpoint —
so the practical blast radius is low but the CI audit noise is high and the
panic advisory (CVSS 5.9) is not hypothetical.

## Decision

Switch the workspace `serenity` feature set from `rustls_backend` to
`native_tls_backend`. This makes `tau-discord-runtime` link against the
operating system's TLS implementation:

- macOS → SecureTransport (Security.framework)
- Linux → OpenSSL (via `native-tls` crate)
- Windows → Schannel

This removes the `rustls 0.22.4` / `rustls-webpki 0.102.8` subtree entirely
from the workspace's `cargo tree` for `tau-discord-runtime`. The only
remaining `rustls-webpki` in the graph is `0.103.13` via the project's
direct `reqwest`/`hyper-rustls` stack, which is unaffected by the advisories.

## Consequences

### Positive
- Four CVEs cleared from `cargo audit`.
- Reduces attack surface by removing an entire transitively-pinned TLS stack.
- No workspace source changes — a one-line Cargo.toml feature flip.
- No runtime API surface change: `serenity`'s public API is identical across
  its `*_backend` features.

### Negative
- Introduces a platform-native TLS build dependency for the discord runtime:
  - Linux CI and production hosts must have OpenSSL headers
    (`libssl-dev` on Debian/Ubuntu, already present on our standard images).
  - macOS uses SecureTransport, which Apple has deprecated in favor of
    Network.framework — long-term this may require another migration.
- OS TLS trust roots differ from `webpki-roots`. For `discord.com` this is
  a non-issue (publicly trusted), but any future private-PKI scenario would
  need rethinking.

### Neutral
- We continue to use `rustls` elsewhere in the workspace (reqwest, hyper, the
  provider/inference stack). This decision is scoped to the discord runtime only.

## Alternatives considered

1. **Ignore the advisories in `.cargo/audit.toml` with a rationale.** Rejected
   as a sole strategy because the panic advisory is concretely exploitable
   under attacker control of the TLS peer (not our case for Discord, but the
   ignore-list is where we keep *un-exploitable* warnings, and these are
   different in kind). Documenting alone would hide a reachable panic path.
2. **Fork `serenity` and bump its `rustls` dep internally.** Rejected —
   maintenance burden, compatibility risk, and serenity upstream is active so
   a fix is likely imminent.
3. **Remove `tau-discord-runtime` entirely.** Rejected — kills a supported
   runtime target.

## References

- [RUSTSEC-2026-0104](https://rustsec.org/advisories/RUSTSEC-2026-0104)
- [RUSTSEC-2026-0098](https://rustsec.org/advisories/RUSTSEC-2026-0098)
- [RUSTSEC-2026-0099](https://rustsec.org/advisories/RUSTSEC-2026-0099)
- [RUSTSEC-2026-0049](https://rustsec.org/advisories/RUSTSEC-2026-0049)
- [`.cargo/audit.toml`](../../.cargo/audit.toml) — scope of what the project accepts as low-risk advisories.
