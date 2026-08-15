//! The knights journey is a puzzle. A knight starts in the upper left corner (A1) of a chessboard
//! and visits each field exactly once.

use generic_backtracking::Solutions;
use journey::Journey;
use position::Position;

mod board;
mod journey;
mod position;

/// Number of solutions to print
const NUM_SOLUTIONS: usize = 1000;

fn main() {
    let start = Position::new(0, 0);
    let journey = Journey::new(start);
    for (index, solution) in Solutions::new(journey).enumerate().take(NUM_SOLUTIONS) {
        let num_solution = index + 1;
        println!("#{num_solution}: {solution}")
    }
}
