# M337 - Browser arcade examples

Status: Active

## Context
Tau keeps a small set of checked-in browser examples under `examples/` so
contributors can inspect, serve, and iterate on self-contained demo slices
without a build pipeline. This milestone groups lightweight browser-game and
interactive showcase work that is intentionally isolated from the Rust runtime.

## Issue Hierarchy
- Local task: `#3748` Add `examples/pacman-tetris-ws3` Phaser mashup example

## Scope
- add or refine static browser examples under `examples/`
- keep examples self-contained and easy to serve locally
- document controls, rules, and run instructions alongside each slice

## Exit Criteria
- each example in scope can be served locally with a simple static server
- example docs tell contributors what the slice demonstrates and how to run it
- browser examples remain isolated from production runtime code

## Delivery Notes
- prefer standalone HTML + JS slices over toolchain-heavy setups
- keep example mechanics understandable from on-page copy and HUD feedback
- favor distinct demos over near-duplicate renames
