use std::fmt::{self, Display, Formatter};

use backtracking::{Problem, Solutions};

fn main() {
    let board = NQueens::new(8);
    for solution in Solutions::new(board) {
        println!("{solution}")
    }
}

#[derive(Clone)]
struct NQueens {
    n: u32,
}

impl NQueens {
    fn new(n: u32) -> Self {
        Self { n }
    }
}

/// Possition of an individual queen on the board
#[derive(Clone, Copy)]
struct QueenAt {
    row: u32,
    column: u32,
}

impl QueenAt {
    /// True if the two queens are not allowed at the board at the same time.
    fn conflicts(self, other: QueenAt) -> bool {
        self.row == other.row
            || self.column == other.column
            || self.row.abs_diff(other.row) == self.column.abs_diff(other.column)
    }
}

impl Problem for NQueens {
    type Possibility = QueenAt;
    type Solution = NQueensSolution;

    fn extend_possibilities(&self, possible_moves: &mut Vec<QueenAt>, history: &[QueenAt]) {
        if history.len() == self.n as usize {
            return;
        }
        // Give all possible position for the top empty row
        let possibilities = (0..self.n)
            .map(|col| QueenAt {
                row: history.len() as u32,
                column: col,
            })
            .filter(|candidate| history.iter().all(|q| !q.conflicts(*candidate)));
        possible_moves.extend(possibilities);
    }

    fn undo(&mut self, _last: &Self::Possibility, _history: &[Self::Possibility]) {}

    fn what_if(&mut self, _next: QueenAt) {}

    fn is_solution(&self, history: &[QueenAt]) -> Option<NQueensSolution> {
        if history.len() == self.n as usize {
            let mut solution = vec![0; self.n as usize];
            for queen in history {
                solution[queen.row as usize] = queen.column;
            }
            Some(NQueensSolution(solution))
        } else {
            None
        }
    }
}

/// Solution to the n queens problem. Nth index of vec contains column index of queen in n-th row.
struct NQueensSolution(Vec<u32>);

impl Display for NQueensSolution {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let repeat_point = |f: &mut Formatter, n| {
            for _ in 0..n {
                write!(f, ".")?;
            }
            Ok(())
        };

        for &pos in &self.0 {
            repeat_point(f, pos)?;
            write!(f, "Q")?;
            repeat_point(f, self.0.len() as u32 - pos - 1)?;
            writeln!(f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_row_conflicts() {
        let a = QueenAt { row: 2, column: 3 };
        let b = QueenAt { row: 2, column: 5 };
        assert!(a.conflicts(b));
    }

    #[test]
    fn same_column_conflicts() {
        let a = QueenAt { row: 1, column: 4 };
        let b = QueenAt { row: 3, column: 4 };
        assert!(a.conflicts(b));
    }

    #[test]
    fn same_diagonal_conflicts() {
        let a = QueenAt { row: 1, column: 1 };
        let b = QueenAt { row: 3, column: 3 };
        assert!(a.conflicts(b));

        let c = QueenAt { row: 0, column: 4 };
        let d = QueenAt { row: 2, column: 2 };
        assert!(c.conflicts(d));
    }

    #[test]
    fn non_conflicting_positions_do_not_conflict() {
        let a = QueenAt { row: 0, column: 0 };
        let b = QueenAt { row: 1, column: 2 };
        assert!(!a.conflicts(b));
    }

    #[test]
    fn extend_possibilities_excludes_conflicting_columns() {
        let queens = NQueens::new(4);
        let history = [QueenAt { row: 0, column: 0 }];

        let mut possibilities = Vec::new();
        queens.extend_possibilities(&mut possibilities, &history);

        // Row 1 candidates must avoid column 0 (same column) and column 1 (diagonal).
        let columns: Vec<u32> = possibilities.iter().map(|q| q.column).collect();
        assert_eq!(vec![2, 3], columns);
        assert!(possibilities.iter().all(|q| q.row == 1));
    }

    #[test]
    fn extend_possibilities_empty_once_every_row_is_placed() {
        let queens = NQueens::new(4);
        let history = vec![QueenAt { row: 0, column: 0 }; 4];

        let mut possibilities = Vec::new();
        queens.extend_possibilities(&mut possibilities, &history);

        assert!(possibilities.is_empty());
    }

    #[test]
    fn is_solution_maps_row_to_column() {
        let queens = NQueens::new(4);
        let history = [
            QueenAt { row: 0, column: 1 },
            QueenAt { row: 1, column: 3 },
            QueenAt { row: 2, column: 0 },
            QueenAt { row: 3, column: 2 },
        ];

        let solution = queens.is_solution(&history).unwrap();
        assert_eq!(vec![1, 3, 0, 2], solution.0);
    }

    #[test]
    fn four_queens_has_exactly_two_solutions() {
        let solutions: Vec<_> = Solutions::new(NQueens::new(4)).collect();
        assert_eq!(2, solutions.len());
    }
}
