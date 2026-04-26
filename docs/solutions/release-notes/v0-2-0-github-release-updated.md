---
title: v0.2.0 GitHub release updated
category: release-notes
date: '2026-04-26'
tags:
  - release
  - github-release
  - v0.2.0
  - product-proof
  - gh-release-edit
related:
  - docs/solutions/release-notes/product-proof-readiness.md
  - CHANGELOG.md
---

# v0.2.0 GitHub release updated
## Problem
The v0.2.0 tag already had a non-draft GitHub release, but the public release body was a large generated changelog and did not reflect the checked-in Product Proof Readiness release notes selected for publication.
## Root cause
The release publication boundary had already been crossed before this stage, so a normal gh release create flow would have failed or duplicated intent. The existing release needed an explicit in-place update rather than tag movement or a second publication attempt.
## Solution
Verified gh authentication, confirmed the pushed v0.2.0 tag target was unchanged, confirmed the existing GitHub release was non-draft and non-prerelease, then ran `gh release edit v0.2.0 --title "v0.2.0 - Product Proof Readiness" --notes-file docs/solutions/release-notes/product-proof-readiness.md`. Follow-up verification confirmed the release title, Product Proof body text, `What's New`, `--webchat-smoke`, body size under 20000 characters, and unchanged remote tag target.
## Prevention

For future releases, check `gh release view <tag>` before `gh release create <tag>`. If a release exists, decide explicitly whether to replace, append, or leave the public body unchanged before running `gh release edit`. Keep the checked-in release notes source concise enough to use directly as the GitHub release body.
