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
