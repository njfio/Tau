# Spec: Issue #3733 - Credential store KDF hardening + legacy cipher migration

Status: Reviewed

## Problem Statement
The credential store currently derives keyed encryption material with a single
SHA-256 hash, permits short 8-character passphrases, and still accepts legacy
v1 payloads that rely on a weaker XOR-based cipher. Tau needs a stronger keyed
envelope format plus a safe migration path so existing stores remain readable
while new writes move to the hardened format.

## Scope
In scope:
- `crates/tau-provider/Cargo.toml`
- `crates/tau-provider/src/credential_store.rs`
- `specs/3733/spec.md`
- `specs/3733/plan.md`
- `specs/3733/tasks.md`

Out of scope:
- changing provider auth UX or secret-store CLI flags
- changing plaintext (`CredentialStoreEncryptionMode::None`) behavior
- rotating or deleting existing credential entries during migration

## Acceptance Criteria
### AC-1 New keyed payloads use a v3 Argon2id-based envelope
Given a keyed credential-store write,
when Tau encrypts a secret,
then it emits an `enc:v3:` payload derived via Argon2id rather than the legacy
single-hash KDF path.

### AC-2 Existing keyed payloads remain readable
Given an existing `enc:v2:` or `enc:v1:` keyed payload,
when Tau decrypts the credential,
then it still resolves successfully with the correct plaintext.

### AC-3 Legacy keyed payloads auto-migrate to v3 on load
Given a credential store contains legacy keyed payloads,
when Tau loads the store successfully,
then it rewrites those keyed entries to the v3 envelope so the store is
persisted in the hardened format after the read.

### AC-4 Hardened keyed writes require a 16-character minimum key
Given an explicit credential-store key is provided for a v3 keyed write,
when the key is shorter than 16 characters,
then Tau rejects the write.

## Conformance Cases
- C-01 / AC-1 / Regression: v3 keyed material differs from the old v2 KDF and
  encrypted payloads use the `enc:v3:` prefix.
- C-02 / AC-2 / Regression: v2 payloads still decrypt successfully.
- C-03 / AC-3 / Regression: loading a store with a legacy v1 keyed payload
  rewrites that payload to v3.
- C-04 / AC-4 / Regression: keyed writes reject keys shorter than 16
  characters.

## Success Metrics / Observable Signals
- New keyed secrets are persisted in the hardened v3 format.
- Old keyed stores remain readable during upgrade.
- Successful reads migrate legacy keyed payloads forward without manual user
  steps.
