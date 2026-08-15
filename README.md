# Backtracking

[![Test](https://github.com/miscelleanous-projs/backtracking/actions/workflows/test.yml/badge.svg)](https://github.com/miscelleanous-projs/backtracking/actions/workflows/test.yml)

Generic implementation of backtracking together with some examples. This came out of a code dojo
session with some of my colleagues (<https://github.com/pacman82/cobra-kai-code-dojo>). I found it
neat enough to put it into its own repostiory.

---

## Fork additions

This fork adds test coverage (the library and all three examples now have unit tests; CI runs
`cargo test --all-targets` against both the default and `parallel` feature sets) and two
backward-compatible extensions to the `Problem`/`Solutions` API:

- **`Problem::Possibility` now requires only `Clone`, not `Copy`.** Every `Copy` type is already
  `Clone`, so existing implementations are unaffected; the weaker bound additionally allows
  heap-backed decision types (an owned `String`, a `Vec`, ...) without forcing them through an
  artificial handle/index indirection just to satisfy the trait.
- **An opt-in `parallel` feature adds `parallel_solutions`.** It searches the independent branches
  rooted at each top-level possibility in parallel (via `rayon`), with each branch still walked
  sequentially by an ordinary `Solutions` iterator on its own thread. The extra `Clone + Send +
  Sync` bounds only apply at that new call site — `Solutions::new` and its bounds are untouched,
  and the `rayon` dependency is not compiled unless the feature is enabled.
