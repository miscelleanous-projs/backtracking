use std::fmt::{self, Display, Formatter};

use super::board::COLUMNS;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Position {
    index: i8,
}

impl Position {
    pub fn from_index(index: usize) -> Self {
        Self {
            index: index.try_into().unwrap(),
        }
    }

    pub fn new(row: i8, column: i8) -> Self {
        Self {
            index: row * COLUMNS as i8 + column,
        }
    }

    pub fn as_index(self) -> usize {
        self.index as usize
    }

    pub fn row(self) -> i8 {
        self.index / COLUMNS as i8
    }

    pub fn column(self) -> i8 {
        self.index % COLUMNS as i8
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let letter = (self.row() as u8 + b'A') as char;
        let digit = (self.column() as u8) + 1;
        write!(f, "{letter}{digit}")
    }
}

#[cfg(test)]
mod tests {
    use super::Position;

    #[test]
    fn print_position() {
        assert_eq!("A1", Position::new(0, 0).to_string());
    }

    /// `(0, 0)` is the one position where mixing up row and column is invisible, so pin the
    /// mapping somewhere off the diagonal too: the row picks the letter, the column the digit.
    #[test]
    fn print_position_off_the_diagonal() {
        assert_eq!("C6", Position::new(2, 5).to_string());
    }

    #[test]
    fn row_and_column_survive_the_round_trip_through_an_index() {
        let position = Position::new(2, 5);

        assert_eq!(2, position.row());
        assert_eq!(5, position.column());
        assert_eq!(position, Position::from_index(position.as_index()));
    }
}
