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
    type Posibility = QueenAt;
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

    fn undo(&mut self, _last: &Self::Posibility, _history: &[Self::Posibility]) {}

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
