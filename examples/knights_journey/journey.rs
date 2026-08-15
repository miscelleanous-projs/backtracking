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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_journey_marks_start_as_visited() {
        let start = Position::new(0, 0);
        let journey = Journey::new(start);

        let mut possible_moves = Vec::new();
        journey.extend_possibilities(&mut possible_moves, &[]);

        assert!(!possible_moves.contains(&start));
    }

    #[test]
    fn what_if_updates_current_and_marks_visited() {
        let start = Position::new(0, 0);
        let mut journey = Journey::new(start);
        let next = Position::new(2, 1);

        journey.what_if(next);

        assert_eq!(next, journey.current);
        assert!(journey.visited[next.as_index()]);
    }

    #[test]
    fn undo_restores_current_to_previous_history_entry_and_clears_visited() {
        let start = Position::new(0, 0);
        let mut journey = Journey::new(start);
        let first = Position::new(2, 1);
        let second = Position::new(4, 2);

        journey.what_if(first);
        journey.what_if(second);
        journey.undo(&second, &[first]);

        assert_eq!(first, journey.current);
        assert!(!journey.visited[second.as_index()]);
    }

    #[test]
    fn undo_falls_back_to_start_when_history_is_empty() {
        let start = Position::new(0, 0);
        let mut journey = Journey::new(start);
        let first = Position::new(2, 1);

        journey.what_if(first);
        journey.undo(&first, &[]);

        assert_eq!(start, journey.current);
    }

    #[test]
    fn is_solution_only_once_every_remaining_field_has_a_move() {
        let start = Position::new(0, 0);
        let journey = Journey::new(start);

        assert!(journey.is_solution(&[]).is_none());

        let history = vec![Position::new(0, 0); NUM_FIELDS - 1];
        assert!(journey.is_solution(&history).is_some());
    }
}
