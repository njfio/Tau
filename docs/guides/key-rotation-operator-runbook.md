# Key Rotation Operator Runbook

## Scope

This runbook covers rotating the encryption key used for Tau credential store
operations without printing or committing secret values. It applies to encrypted
credential stores used by provider and integration auth flows.

## Preconditions

- Identify the credential store path and encryption mode from the operator
  launch configuration.
- Confirm the current key source is available to the operator session.
- Confirm the replacement key source is available but not written to shell
  history, logs, issues, or PRs.
- Stop or pause automation that may mutate the credential store during rotation.
- Create an access-controlled backup of the encrypted credential store file.

## Rotate

1. Load the store with the current key in an isolated operator shell.
2. Export only non-secret inventory metadata: credential ids, provider names,
   revoked flags, and updated timestamps.
3. Re-save the store with the replacement key using the existing credential
   store writer path.
4. Clear any process environment that held the old key.
5. Restart only the local processes that need to read the new key source.

Do not copy plaintext credential values into notes, shell transcripts, issue
comments, or generated reports.

## Verify

Run the credential lifecycle verification for the target environment:

```bash
scripts/verify/m309-auth-credential-lifecycle-depth.sh
```

Then verify at least one non-mutating provider/integration auth read path using
the new key source. Expected signals:

- existing credential ids are still discoverable;
- revoked entries remain revoked;
- missing credential errors remain deterministic;
- no logs contain credential values or raw key material.

## Rollback

Rollback is allowed only before new credentials have been written with the new
key.

1. Stop readers/writers of the rotated store.
2. Restore the access-controlled encrypted backup.
3. Restore the previous key source for the affected local processes.
4. Re-run the credential lifecycle verification.
5. Record the rollback reason without including secret material.

If new credentials were written after rotation, do not restore the old encrypted
file blindly. Export metadata, identify changed credential ids, and create a
follow-up issue for a controlled merge/restore plan.

## Release Evidence

For release review, record:

- store path pattern, not the secret path if it exposes user names or tokens;
- verification commands and pass/fail status;
- affected providers/integrations;
- rollback status: `not-needed`, `completed`, or `blocked`;
- confirmation that no secret values were logged.
