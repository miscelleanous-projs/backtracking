//! Find solutions to combinatorial search problems using backtracking.
//!
//! Implement [`Problem`] to describe how a problem's decisions are made, then drive it with the
//! [`Solutions`] iterator. This example enumerates every subset of `[1, 2, 3]` that sums to `3`:
//!
//! ```
//! use generic_backtracking::{Problem, Solutions};
//!
//! struct SubsetSum {
//!     items: Vec<i32>,
//!     target: i32,
//! }
//!
//! impl Problem for SubsetSum {
//!     /// Index of an item we decide to include next.
//!     type Possibility = usize;
//!     /// The subset of items which sums to `target`.
//!     type Solution = Vec<i32>;
//!
//!     fn extend_possibilities(&self, possibilities: &mut Vec<usize>, history: &[usize]) {
//!         // Only consider items after the last one picked, so each subset is generated once.
//!         let next = history.last().map_or(0, |&i| i + 1);
//!         possibilities.extend(next..self.items.len());
//!     }
//!
//!     fn what_if(&mut self, _decision: usize) {}
//!     fn undo(&mut self, _last: &usize, _history: &[usize]) {}
//!
//!     fn is_solution(&self, history: &[usize]) -> Option<Vec<i32>> {
//!         let subset: Vec<i32> = history.iter().map(|&i| self.items[i]).collect();
//!         (subset.iter().sum::<i32>() == self.target).then_some(subset)
//!     }
//! }
//!
//! let problem = SubsetSum { items: vec![1, 2, 3], target: 3 };
//! let mut solutions: Vec<Vec<i32>> = Solutions::new(problem).collect();
//! solutions.sort();
//! assert_eq!(solutions, vec![vec![1, 2], vec![3]]);
//! ```

/// A problem to be tackled with backtracking. Used by the [`Solutions`] iterator which can find
/// solutions for types implementing [`Problem`].
///
/// Technically any problem solvable with backtracking would not need to keep any state, apart from
/// the initial state, since all the essential input is part of the history. An empty implementation
/// for [`Problem::what_if`] and [`Problem::undo`] would always be sufficient. Given the large
/// search space for many of these problems, though, real world implementation are likely to keep
/// some cached state, which is updated in these methods.
pub trait Problem {
    /// Describes a decision made in a problem state leading to a new candidate for a solution. E.g.
    /// which field to jump to in a knights journey problem or which digit to write into a cell for
    /// a sudoku puzzle.
    type Possibility: Clone;
    /// Final state we are interested in. E.g. The history of moves made for a knights journey, or
    /// the final distribution of digits in the cells of a sudoku puzzle.
    type Solution;

    /// Extends `possibilities` with a set of decisions to be considered next. Implementations may
    /// assume that the `possibilities` is empty if invoked through the `Solutions` iterator.
    fn extend_possibilities(
        &self,
        possibilities: &mut Vec<Self::Possibility>,
        history: &[Self::Possibility],
    );

    /// Undo the last decision made. If invoked by the [`Solutions`] iterator `last` is to be
    /// guaranteed, to be the last decision made with [`what_if`](Problem::what_if)
    fn undo(&mut self, last: &Self::Possibility, history: &[Self::Possibility]);

    /// Update internal caches to reflect a scenario in which we would decide to execute the given
    /// possibility.
    fn what_if(&mut self, decision: Self::Possibility);

    /// Check if the candidate state we are looking at is a solution to our problem. If so extract
    /// the information we are interested in.
    fn is_solution(&self, history: &[Self::Possibility]) -> Option<Self::Solution>;
}

/// An iterator performing backtracking to find solutions to a problem.
pub struct Solutions<P: Problem> {
    decisions: Vec<P::Possibility>,
    open: Vec<Candidate<P::Possibility>>,
    /// Keeps track of the decisions, which yielded the current problem state, starting from the
    /// initial state.
    history: Vec<P::Possibility>,
    current: P,
}

impl<G: Problem> Solutions<G> {
    pub fn new(init: G) -> Self {
        let mut possible_moves = Vec::new();
        init.extend_possibilities(&mut possible_moves, &[]);
        let open = possible_moves
            .iter()
            .map(|pos| Candidate {
                count: 1,
                possibility: pos.clone(),
            })
            .collect();
        Self {
            decisions: possible_moves,
            open,
            history: Vec::new(),
            current: init,
        }
    }

    /// Restricts the initial search frontier to a single possibility, leaving the rest of the
    /// tree walk identical to [`Solutions::new`]. Used to partition the root of the search tree
    /// into independent branches, e.g. for [`parallel_solutions`].
    #[cfg(feature = "parallel")]
    fn seeded(current: G, first: G::Possibility) -> Self {
        Self {
            decisions: Vec::new(),
            open: vec![Candidate {
                count: 1,
                possibility: first,
            }],
            history: Vec::new(),
            current,
        }
    }

    /// Unwinds `history`/`current` until `history.len()` equals `target_depth - 1` — one move
    /// behind the candidate about to be played at `target_depth`. This simulates the stack frames
    /// a recursive implementation would have unwound automatically; isolated here to keep that
    /// concern separate from the rest of `next()`'s algorithm skeleton.
    #[inline]
    fn rewind_to(&mut self, target_depth: i32) {
        for _ in 0..self.history.len() as i32 - target_depth + 1 {
            let last = self.history.pop().unwrap();
            self.current.undo(&last, &self.history);
        }
    }
}

/// **Experimental.** Explores the independent branches rooted at each of the initial
/// possibilities of `init` in parallel, each branch searched sequentially by an ordinary
/// [`Solutions`] iterator on its own thread. Requires the `parallel` feature.
///
/// This only requires [`Clone`] on `P` (to fork one problem instance per root branch) and
/// [`Send`] bounds (to move those instances across threads); it does not change [`Problem`] or
/// [`Solutions`] in any way, so it is purely additive to the crate's API.
///
/// Unlike the core [`Problem`]/[`Solutions`] API, this entry point is newer, isn't exercised by
/// any of the bundled examples, and its API shape may still change in a minor version bump.
#[cfg(feature = "parallel")]
pub fn parallel_solutions<P>(init: P) -> impl rayon::iter::ParallelIterator<Item = P::Solution>
where
    P: Problem + Clone + Send + Sync,
    P::Solution: Send,
    P::Possibility: Send,
{
    use rayon::iter::{IntoParallelIterator, ParallelIterator};

    let mut roots = Vec::new();
    init.extend_possibilities(&mut roots, &[]);

    roots.into_par_iter().flat_map_iter(move |first_move| {
        Solutions::seeded(init.clone(), first_move)
    })
}

impl<G: Problem> Iterator for Solutions<G> {
    type Item = G::Solution;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(Candidate {
            count,
            possibility: mov,
        }) = self.open.pop()
        {
            // Unroll all the moves until our current state is identical with the one which we
            // had once we put that mov into the open list. We want to be one move behind so
            // we need to play the move in order to get the desired state
            self.rewind_to(count);

            // We advance one move deeper into the search tree
            self.current.what_if(mov.clone());
            self.history.push(mov);

            // Emit solution
            if let Some(solution) = self.current.is_solution(&self.history) {
                return Some(solution);
            }

            // Extend search tree
            self.decisions.clear();
            self.current
                .extend_possibilities(&mut self.decisions, &self.history);
            self.open
                .extend(self.decisions.iter().map(|position| Candidate {
                    count: count + 1,
                    possibility: position.clone(),
                }))
        }
        None
    }
}

struct Candidate<P> {
    /// Counts the number of turns made to get to this candidate. We keep track of this so we can
    /// call undo the appropriate number of times, if we roll back to an earlier state.
    count: i32,
    /// Possibility which will lead to this candidate
    possibility: P,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Shared by every test `Problem` below that models `n` independent binary choices: extend
    /// with both `false` and `true` until `n` decisions have been made.
    fn extend_with_binary_choices(possibilities: &mut Vec<bool>, history: &[bool], n: usize) {
        if history.len() < n {
            possibilities.push(false);
            possibilities.push(true);
        }
    }

    /// Enumerates every binary sequence of length `n`, without any pruning. Used to check that
    /// `Solutions` visits the entire search tree.
    struct BinarySequences {
        n: usize,
    }

    impl Problem for BinarySequences {
        type Possibility = bool;
        type Solution = Vec<bool>;

        fn extend_possibilities(&self, possibilities: &mut Vec<bool>, history: &[bool]) {
            extend_with_binary_choices(possibilities, history, self.n);
        }

        fn undo(&mut self, _last: &bool, _history: &[bool]) {}

        fn what_if(&mut self, _decision: bool) {}

        fn is_solution(&self, history: &[bool]) -> Option<Vec<bool>> {
            (history.len() == self.n).then(|| history.to_vec())
        }
    }

    #[test]
    fn enumerates_all_binary_sequences() {
        let solutions: Vec<_> = Solutions::new(BinarySequences { n: 3 }).collect();

        assert_eq!(8, solutions.len());
        let mut distinct = solutions;
        distinct.sort();
        distinct.dedup();
        assert_eq!(8, distinct.len());
    }

    #[test]
    fn no_solutions_if_extend_possibilities_never_yields_candidates() {
        struct Empty;

        impl Problem for Empty {
            type Possibility = ();
            type Solution = ();

            fn extend_possibilities(&self, _possibilities: &mut Vec<()>, _history: &[()]) {}
            fn undo(&mut self, _last: &(), _history: &[()]) {}
            fn what_if(&mut self, _decision: ()) {}
            fn is_solution(&self, _history: &[()]) -> Option<()> {
                None
            }
        }

        let solutions: Vec<_> = Solutions::new(Empty).collect();

        assert!(solutions.is_empty());
    }

    /// Only yields permutations of `0..n`, by refusing to reuse a value already present in
    /// `history`. Checks that backtracking correctly prunes branches based on sibling state.
    #[derive(Clone)]
    struct DistinctPermutations {
        n: u8,
    }

    impl Problem for DistinctPermutations {
        type Possibility = u8;
        type Solution = Vec<u8>;

        fn extend_possibilities(&self, possibilities: &mut Vec<u8>, history: &[u8]) {
            if history.len() == self.n as usize {
                return;
            }
            possibilities.extend((0..self.n).filter(|value| !history.contains(value)));
        }

        fn undo(&mut self, _last: &u8, _history: &[u8]) {}

        fn what_if(&mut self, _decision: u8) {}

        fn is_solution(&self, history: &[u8]) -> Option<Vec<u8>> {
            (history.len() == self.n as usize).then(|| history.to_vec())
        }
    }

    #[test]
    fn pruned_search_only_yields_permutations() {
        let solutions: Vec<_> = Solutions::new(DistinctPermutations { n: 3 }).collect();

        // 3! permutations of [0, 1, 2]
        assert_eq!(6, solutions.len());
        for permutation in &solutions {
            let mut sorted = permutation.clone();
            sorted.sort();
            assert_eq!(vec![0, 1, 2], sorted);
        }
    }

    #[cfg(feature = "parallel")]
    #[test]
    fn parallel_search_yields_the_same_solutions_as_sequential_search() {
        use rayon::iter::ParallelIterator;

        let mut sequential: Vec<_> = Solutions::new(DistinctPermutations { n: 4 }).collect();
        let mut parallel: Vec<_> = parallel_solutions(DistinctPermutations { n: 4 }).collect();

        sequential.sort();
        parallel.sort();
        assert_eq!(sequential, parallel);
        assert_eq!(24, parallel.len()); // 4! permutations
    }

    /// Tracks a running sum as cached state via `what_if`/`undo`, and cross checks it against a
    /// sum computed fresh from `history` on every candidate solution. This exercises that
    /// `Solutions` unwinds (`undo`s) and replays (`what_if`s) the cache correctly when
    /// backtracking between sibling branches, not just on the way down.
    struct SumChecking {
        n: usize,
        cached_sum: i32,
    }

    impl Problem for SumChecking {
        type Possibility = bool;
        type Solution = i32;

        fn extend_possibilities(&self, possibilities: &mut Vec<bool>, history: &[bool]) {
            extend_with_binary_choices(possibilities, history, self.n);
        }

        fn undo(&mut self, last: &bool, _history: &[bool]) {
            if *last {
                self.cached_sum -= 1;
            }
        }

        fn what_if(&mut self, decision: bool) {
            if decision {
                self.cached_sum += 1;
            }
        }

        fn is_solution(&self, history: &[bool]) -> Option<i32> {
            if history.len() == self.n {
                let expected = history.iter().filter(|&&bit| bit).count() as i32;
                assert_eq!(
                    expected, self.cached_sum,
                    "cached sum diverged from history"
                );
                Some(self.cached_sum)
            } else {
                None
            }
        }
    }

    #[test]
    fn cached_state_stays_in_sync_with_history_across_backtracks() {
        let solutions: Vec<_> = Solutions::new(SumChecking { n: 4, cached_sum: 0 }).collect();

        assert_eq!(16, solutions.len());
    }
}
