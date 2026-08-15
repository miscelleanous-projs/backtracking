use std::fmt::{self, Display, Formatter};

use backtracking::Problem;

use super::{
    board::{Board, NUM_FIELDS},
    position::Position,
};

#[derive(Clone, Debug)]
pub struct Journey {
    board: Board,
    /// For fast lookup, wether a position has been visited or not.
    visited: [bool; NUM_FIELDS],
    /// Currenty position of the knight
    current: Position,
    /// Starting position
    start: Position,
}

impl Journey {
    pub fn new(start: Position) -> Self {
        let mut visited = [false; NUM_FIELDS];
        visited[start.as_index()] = true;
        Self {
            board: Board::new(),
            visited,
            current: start,
            start,
        }
    }
}

impl Display for Solution {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0[0])?;
        for m in &self.0[1..NUM_FIELDS] {
            write!(f, " {m}")?;
        }
        Ok(())
    }
}

impl Problem for Journey {
    type Posibility = Position;
    type Solution = Solution;

    fn extend_possibilities(&self, possible_moves: &mut Vec<Position>, _history: &[Position]) {
        self.board.reachable_fields(self.current, possible_moves);
        possible_moves.retain(|pos| !self.visited[pos.as_index()])
    }

    fn undo(&mut self, last: &Position, history: &[Position]) {
        self.current = history.last().copied().unwrap_or(self.start);
        self.visited[last.as_index()] = false;
    }

    fn what_if(&mut self, next: Position) {
        self.current = next;
        self.visited[next.as_index()] = true;
    }

    fn is_solution(&self, history: &[Position]) -> Option<Solution> {
        if history.len() == NUM_FIELDS - 1 {
            let mut moves = [self.start; NUM_FIELDS];
            moves[1..].copy_from_slice(history);
            Some(Solution(moves))
        } else {
            None
        }
    }
}

pub struct Solution([Position; NUM_FIELDS]);
