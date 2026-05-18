# Security Policy

## Supported Versions

Security fixes are prioritized for the current `master` branch and the most recent release artifacts produced from it.

## Reporting Channels

Do not open public GitHub issues for suspected vulnerabilities.

Use one of the following private channels:

1. GitHub Security Advisories (preferred): open a private advisory in this repository.
2. Maintainer private contact path (fallback): if advisory flow is unavailable, request private maintainer contact and include full details.

Include the following in your report:

- Vulnerability description and impact
- Reproduction steps or proof-of-concept
- Suspected affected components/crates
- Commit/version context
- Suggested mitigation (if known)

## Triage and Response SLA

- Initial triage target: within 3 business days.
- Severity and scope confirmation follows triage.
- Maintainers provide remediation plan and expected fix window after confirmation.

## Coordinated Disclosure

Please allow time for a patch before public disclosure.

When a fix is ready:

- Patch PR/release is prepared.
- Advisory/changelog notes describe affected versions and upgrade path.
- Reporter credit is provided on request.

## Release Freshness Review

Before each release branch or release candidate, security reviewers should:

- Confirm supported-version wording matches the actual release target.
- Review dependency advisories and record any accepted risk with an owner.
- Re-run credential lifecycle verification when auth, provider, or credential
  store code changed.
- Confirm key-rotation guidance in
  `docs/guides/key-rotation-operator-runbook.md` still matches current
  credential store behavior.
- Verify PRs and release notes do not expose secret values, key material, or
  private vulnerability details.
