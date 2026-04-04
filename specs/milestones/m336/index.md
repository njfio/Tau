# M336 - Browser game examples

Status: Active

## Context
The repository already ships checked-in examples for packages, extensions, and
events workflows, but it does not yet include a self-contained browser game
example. Adding a playable browser example gives the examples area a concrete
greenfield artifact that can exercise repo skills around Phaser/web-game work
without touching the Rust product surface.

## Issue Hierarchy
- Task: [#3745](https://github.com/njfio/Tau/issues/3745) Add a Pac-Man +
  Tetris Phaser example under `examples/`
- Task: [#3746](https://github.com/njfio/Tau/issues/3746) Add a sibling
  `pacman-tetris-ws` Phaser example under `examples/`

## Scope
- keep the work isolated to `examples/`
- ship a self-contained Phaser 3 folder that can be served locally
- document the example in the examples indexes

## Exit Criteria
- the repo includes a playable browser-game example under `examples/`
- the example is discoverable from the checked-in docs
- the slice stays isolated from unrelated Rust/runtime modules

## Delivery Notes
- GitHub issue creation was blocked locally because `gh auth status` reported an
  invalid token, so Issue `#3745` is a reserved local spec slot for this slice
  until remote issue creation can be reconciled
- Issue `#3746` is also a reserved local spec slot for the sibling browser-game
  variant because the same invalid `gh` token blocked remote issue creation
