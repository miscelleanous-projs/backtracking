# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

- Build: `cargo build`
- Test (default): `cargo test --all-targets`
- Test (with the opt-in parallel search feature): `cargo test --all-targets --features parallel`
  - Both are run in CI; changes to `src/lib.rs` should pass both.
  - `word_chain` requires the `parallel` feature (`required-features` in `Cargo.toml`), so it's only built/tested by the second command, not the first.
- Run an example: `cargo run --example n_queens`, `cargo run --example sudoku --release`, `cargo run --example knights_journey --release`, `cargo run --example word_chain --release --features parallel`, `cargo run --example hamiltonian_path`
  - (`knights_journey`, `sudoku`, and `word_chain` are slow enough that CI runs them in `--release` mode.)
- Run a single test: `cargo test <test_name>`
- Lint: `cargo clippy --all-targets --features parallel`
- Docs: `cargo doc --no-deps --features parallel` (should build with no `rustdoc` warnings)

CI (`.github/workflows/test.yml`) runs `cargo test --all-targets` against both the default and `parallel` feature sets, plus the `knights_journey` and `sudoku` examples in release mode — treat those two as executable regression tests, not just demos. Unit tests live alongside the code they test (`#[cfg(test)] mod tests` in `src/lib.rs` and in each example file), not in a separate `tests/` directory.

## Architecture

The core library is `src/lib.rs` and consists of one trait, one iterator, and one opt-in parallel entry point:

- **`Problem` trait** — defines a search problem in terms of incremental decisions:
  - `extend_possibilities` — given the current `history` of decisions, appends candidate next decisions.
  - `what_if` — mutates the implementor's internal state to reflect making a decision (used for caching, e.g. marking a board cell occupied).
  - `undo` — reverses a `what_if`, restoring state when backtracking past that decision.
  - `is_solution` — checks whether the current `history` represents a complete solution, returning `Some(Self::Solution)` if so.
  - The associated `Possibility` type only requires `Clone` (not `Copy`), so decisions can be heap-backed (e.g. an owned `String`) as well as small stack values.

- **`Solutions<P: Problem>`** — a lazy `Iterator<Item = P::Solution>` implementing depth-first backtracking search over an implicit tree of `Problem::Possibility` decisions. It maintains:
  - `open`: a stack of `Candidate { count, possibility }` — unexplored decisions paired with their depth in the tree.
  - `history`: the sequence of decisions leading to the current state.
  - `current`: the live `Problem` instance whose internal cache reflects `history`.

  On each `next()`, it pops a candidate, unwinds (`undo`s) `current`/`history` back to that candidate's depth, applies the candidate (`what_if`), checks `is_solution`, and if not a solution, extends `open` with the new frontier via `extend_possibilities`. This unwind-then-replay approach means `Problem` implementations only need to support single-step `what_if`/`undo`, not arbitrary jumps — the iterator handles rewinding the tree walk itself.

- **`parallel_solutions`** (behind the `parallel` Cargo feature, optional `rayon` dependency) — explores the independent branches rooted at each top-level possibility in parallel, each branch still walked sequentially by an ordinary `Solutions` iterator on its own thread. It requires `Problem: Clone + Send + Sync` at the call site only; `Solutions` itself is untouched and has no such bounds. It's an opt-in extension, not part of the core API most `Problem` implementors need; `word_chain` is the one bundled example that uses it.

Correctness of a `Problem` impl hinges entirely on `what_if`/`undo` being exact inverses and `extend_possibilities` assuming `possibilities` starts empty (per its doc comment).

## Examples

`examples/` contains five sample problems implementing `Problem`, useful as reference when writing a new one:

- `n_queens.rs` — single-file example; `what_if`/`undo` are no-ops (conflict checking is done fresh from `history` each time in `extend_possibilities`), showing the simplest possible `Problem` impl.
- `sudoku.rs` — single-file example.
- `knights_journey/` — multi-module example (`main.rs`, `board.rs`, `journey.rs`, `position.rs`) showing a `Problem` that does maintain cached state across `what_if`/`undo`, and demonstrates limiting output via `.take(NUM_SOLUTIONS)` since the full solution space is large.
- `word_chain.rs` — single-file example, gated behind the `parallel` feature. Runs the same search (find every `target_len`-long chain of dictionary words where each word starts with the previous word's last letter) once with `Solutions` and once with `parallel_solutions`, printing both timings so the speedup is visible.
- `hamiltonian_path.rs` — single-file example. Finds Hamiltonian paths on an explicit adjacency-list graph (the Petersen graph) rather than a grid/board, showing `Problem` working over a general graph structure; reuses the cached `visited` state pattern from `knights_journey` but in that more general setting.
