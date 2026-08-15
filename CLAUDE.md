# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

- Build: `cargo build`
- Test: `cargo test --verbose`
- Run an example: `cargo run --example n_queens`, `cargo run --example sudoku --release`, `cargo run --example knights_journey --release`
  - (`knights_journey` and `sudoku` are slow enough that CI runs them in `--release` mode.)
- Run a single test: `cargo test <test_name>`

CI (`.github/workflows/test.yml`) runs `cargo test --verbose` plus the `knights_journey` and `sudoku` examples in release mode — treat those two as executable regression tests, not just demos.

## Architecture

The entire library is `src/lib.rs` and consists of one trait and one iterator:

- **`Problem` trait** — defines a search problem in terms of incremental decisions:
  - `extend_possibilities` — given the current `history` of decisions, appends candidate next decisions.
  - `what_if` — mutates the implementor's internal state to reflect making a decision (used for caching, e.g. marking a board cell occupied).
  - `undo` — reverses a `what_if`, restoring state when backtracking past that decision.
  - `is_solution` — checks whether the current `history` represents a complete solution, returning `Some(Self::Solution)` if so.

- **`Solutions<P: Problem>`** — a lazy `Iterator<Item = P::Solution>` implementing depth-first backtracking search over an implicit tree of `Problem::Possibility` decisions. It maintains:
  - `open`: a stack of `Candidate { count, possibility }` — unexplored decisions paired with their depth in the tree.
  - `history`: the sequence of decisions leading to the current state.
  - `current`: the live `Problem` instance whose internal cache reflects `history`.

  On each `next()`, it pops a candidate, unwinds (`undo`s) `current`/`history` back to that candidate's depth, applies the candidate (`what_if`), checks `is_solution`, and if not a solution, extends `open` with the new frontier via `extend_possibilities`. This unwind-then-replay approach means `Problem` implementations only need to support single-step `what_if`/`undo`, not arbitrary jumps — the iterator handles rewinding the tree walk itself.

Correctness of a `Problem` impl hinges entirely on `what_if`/`undo` being exact inverses and `extend_possibilities` assuming `possibilities` starts empty (per its doc comment).

## Examples

`examples/` contains three sample problems implementing `Problem`, useful as reference when writing a new one:

- `n_queens.rs` — single-file example; `what_if`/`undo` are no-ops (conflict checking is done fresh from `history` each time in `extend_possibilities`), showing the simplest possible `Problem` impl.
- `sudoku.rs` — single-file example.
- `knights_journey/` — multi-module example (`main.rs`, `board.rs`, `journey.rs`, `position.rs`) showing a `Problem` that does maintain cached state across `what_if`/`undo`, and demonstrates limiting output via `.take(NUM_SOLUTIONS)` since the full solution space is large.
