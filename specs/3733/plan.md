# Plan: Issue #3733 - Credential store KDF hardening + legacy cipher migration

## Approach
1. Add a crate-local `argon2` dependency for a v3 keyed KDF.
2. Keep legacy v2 SHA-256 derivation and v1 XOR decryption helpers strictly for
   backward-compatible reads.
3. Introduce a new `enc:v3:` envelope that stores a per-secret salt and AES-GCM
   ciphertext derived from Argon2id key material.
4. Detect legacy keyed payloads during `load_credential_store` and rewrite the
   store once in the v3 format after a successful read.
5. Add focused tests for KDF divergence, v2 backward compatibility, v1
   migration, and the new minimum key length.

## Risks / Mitigations
- Risk: migration could lock users out if a legacy payload no longer decrypts.
  Mitigation: keep explicit v1/v2 decrypt paths and test them directly.
- Risk: rewriting on read could corrupt the store.
  Mitigation: reuse the existing `save_credential_store` atomic-write path and
  only rewrite after a successful full load.
- Risk: stronger passphrase rules could break legacy decryption.
  Mitigation: enforce the 16-character minimum only for v3 writes/KDF, while
  legacy v1/v2 decrypt paths retain their old compatibility threshold.

## Verification
- `cargo test -p tau-provider regression_spec_3733_c01_v3_kdf_differs_from_v2 -- --nocapture`
- `cargo test -p tau-provider regression_spec_3733_c02_v2_payload_still_decrypts -- --nocapture`
- `cargo test -p tau-provider regression_spec_3733_c03_legacy_v1_payload_auto_migrates_to_v3_on_load -- --nocapture`
- `cargo test -p tau-provider regression_spec_3733_c04_min_key_length_16_enforced_for_v3 -- --nocapture`
- `cargo check -p tau-provider`
- `cargo fmt --check`
