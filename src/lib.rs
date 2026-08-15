//! Find solutions with backtracking.

/// A problem to be tackled with backtracking. Used by the [`Solutions`] iterator which can find
/// solutions for ypes implementing [`Problem`].
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
    type Posibility: Copy;
    /// Final state we are interested in. E.g. The history of moves made for a knights journey, or
    /// the final distribution of digits in the cells of a sudoku puzzle.
    type Solution;

    /// Extends `possibilities` with a set of decisions to be considered next. Implementations may
    /// assume that the `possibilities` is empty if invoked through the `Solutions` iterator.
    fn extend_possibilities(
        &self,
        possibilities: &mut Vec<Self::Posibility>,
        history: &[Self::Posibility],
    );

    /// Undo the last decision made. If invoked by the [`Solutions`] iterator `last` is to be
    /// guaranteed, to be the last decision made with [`do`]
    fn undo(&mut self, last: &Self::Posibility, history: &[Self::Posibility]);

    /// Update internal caches to reflect a scenario in which we would decide to execute the given
    /// possibility.
    fn what_if(&mut self, decision: Self::Posibility);

    /// Check if the candidate state we are looking at is a solution to our probelm. If so extract
    /// the information we are interessted in.
    fn is_solution(&self, history: &[Self::Posibility]) -> Option<Self::Solution>;
}

/// An iterator performing backtracking to find solutions to a problem.
pub struct Solutions<P: Problem> {
    decisions: Vec<P::Posibility>,
    open: Vec<Candidate<P::Posibility>>,
    /// Keeps track of the decisions, which yielded the current problem state, starting from the
    /// initial state.
    history: Vec<P::Posibility>,
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
                possibility: *pos,
            })
            .collect();
        Self {
            decisions: possible_moves,
            open,
            history: Vec::new(),
            current: init,
        }
    }
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
            for _ in 0..self.history.len() as i32 - count + 1 {
                let last = self.history.pop().unwrap();
                self.current.undo(&last, &self.history);
            }

            // We advance one move deeper into the search tree
            self.current.what_if(mov);
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
                .extend(self.decisions.iter().map(|&position| Candidate {
                    count: count + 1,
                    possibility: position,
                }))
        }
        None
    }
}

struct Candidate<P> {
    /// Counts the number of turns made to get to this candidate. We keep track of this so we can
    /// call undo the appropriate number of types, if we roll back to an earlier state.
    count: i32,
    /// Possibility which will lead to this candidate
    possibility: P,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Enumerates every binary sequence of length `n`, without any pruning. Used to check that
    /// `Solutions` visits the entire search tree.
    struct BinarySequences {
        n: usize,
    }

    impl Problem for BinarySequences {
        type Posibility = bool;
        type Solution = Vec<bool>;

        fn extend_possibilities(&self, possibilities: &mut Vec<bool>, history: &[bool]) {
            if history.len() < self.n {
                possibilities.push(false);
                possibilities.push(true);
            }
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
            type Posibility = ();
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
    struct DistinctPermutations {
        n: u8,
    }

    impl Problem for DistinctPermutations {
        type Posibility = u8;
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

    /// Tracks a running sum as cached state via `what_if`/`undo`, and cross checks it against a
    /// sum computed fresh from `history` on every candidate solution. This exercises that
    /// `Solutions` unwinds (`undo`s) and replays (`what_if`s) the cache correctly when
    /// backtracking between sibling branches, not just on the way down.
    struct SumChecking {
        n: usize,
        cached_sum: i32,
    }

    impl Problem for SumChecking {
        type Posibility = bool;
        type Solution = i32;

        fn extend_possibilities(&self, possibilities: &mut Vec<bool>, history: &[bool]) {
            if history.len() < self.n {
                possibilities.push(false);
                possibilities.push(true);
            }
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
