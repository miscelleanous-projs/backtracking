use crate::position::Position;

// Dimensions of the chessboard
pub const ROWS: usize = 8;
pub const COLUMNS: usize = 8;
pub const NUM_FIELDS: usize = ROWS * COLUMNS;

#[derive(Clone, Debug)]
pub struct Board {
    /// Cache the reachable fields from each position in the board
    reachable_fields: Vec<Vec<Position>>,
}

impl Board {
    pub fn new() -> Self {
        let reachable_fields = (0..NUM_FIELDS)
            .map(|index| compute_reachable_fields(Position::from_index(index)))
            .collect();
        Self { reachable_fields }
    }

    pub fn reachable_fields(&self, position: Position, possible_moves: &mut Vec<Position>) {
        possible_moves.clear();
        possible_moves.extend(self.reachable_fields[position.as_index()].iter());
    }
}

/// All possible moves, taking into account the position in the board
fn compute_reachable_fields(position: Position) -> Vec<Position> {
    // Possible Moves of the knight
    const MOVES: [(i8, i8); 8] = [
        (-2, -1),
        (-2, 1),
        (-1, -2),
        (-1, 2),
        (1, -2),
        (1, 2),
        (2, -1),
        (2, 1),
    ];

    MOVES
        .iter()
        .map(|(delta_row, delta_column)| {
            (position.row() - delta_row, position.column() - delta_column)
        })
        .filter(|&(r, c)| (0..ROWS as i8).contains(&r) && (0..COLUMNS as i8).contains(&c))
        .map(|(row, column)| Position::new(row, column))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corner_has_two_reachable_fields() {
        let board = Board::new();
        let mut moves = Vec::new();

        board.reachable_fields(Position::new(0, 0), &mut moves);

        assert_eq!(2, moves.len());
    }

    #[test]
    fn center_has_eight_reachable_fields() {
        let board = Board::new();
        let mut moves = Vec::new();

        board.reachable_fields(Position::new(3, 3), &mut moves);

        assert_eq!(8, moves.len());
    }

    #[test]
    fn reachable_fields_clears_prior_contents() {
        let board = Board::new();
        let mut moves = vec![Position::new(7, 7)];

        board.reachable_fields(Position::new(0, 0), &mut moves);

        assert_eq!(2, moves.len());
        assert!(!moves.contains(&Position::new(7, 7)));
    }
}
