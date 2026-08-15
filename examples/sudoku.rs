use std::io::{self, stdout, Write};

use backtracking::{Problem, Solutions};

fn main() -> io::Result<()> {
    // An empty sudoku field
    let sudoku = Sudoku::from_bytes([
        6, 0, 3, 0, 0, 0, 1, 0, 0, 0, 0, 9, 0, 0, 0, 2, 0, 0, 0, 0, 7, 4, 0, 9, 0, 0, 0, 0, 0, 0,
        0, 1, 0, 0, 0, 7, 4, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 5, 3, 0, 1, 0, 0, 0, 0,
        0, 4, 0, 0, 0, 6, 3, 0, 7, 0, 9, 0, 0, 9, 0, 0, 0, 2, 0, 3, 0,
    ]);
    for solution in Solutions::new(sudoku).take(1) {
        solution.print_to(&mut stdout())?
    }
    Ok(())
}

#[derive(Clone)]
pub struct Sudoku {
    /// All 9 by 9 fields, in top to bottom, left to right order. `0` represents empty. Other valid
    /// values are 1..=9
    fields: [u8; 9 * 9],
}

impl Sudoku {
    pub fn new() -> Self {
        let fields = [0u8; 9 * 9];
        Self::from_bytes(fields)
    }

    pub fn from_bytes(bytes: [u8; 9 * 9]) -> Self {
        if bytes.iter().any(|&n| n > 9) {
            panic!("Only values from 0 to 9 are valid.")
        }
        Self { fields: bytes }
    }

    pub fn print_to(&self, to: &mut impl Write) -> io::Result<()> {
        for index in 0..self.fields.len() {
            // New row beginnig?
            if index % 9 == 0 && index != 0 {
                writeln!(to)?;
            }
            match self.fields[index] {
                0 => write!(to, "X")?,
                n @ 1..=9 => write!(to, "{n}")?,
                _ => unreachable!(),
            };
        }
        writeln!(to)?;
        Ok(())
    }

    pub fn possible_digits_at(&self, index: u8) -> impl Iterator<Item = u8> + '_ {
        let row = index as usize / 9;
        let col = index as usize % 9;
        let group = col / 3 + (row / 3) * 3;
        // Index upper right corner of group
        let group_off = group * 3 + (group / 3) * 18;
        let is_in_row = move |digit| (0..9).any(|c| self.fields[c + row * 9] == digit);
        let is_in_col = move |digit| (0..9).any(|r| self.fields[col + r * 9] == digit);
        let is_in_group =
            move |digit| (0..9).any(|i| self.fields[group_off + i % 3 + (i / 3) * 9] == digit);
        (1..=9)
            .filter(move |digit| !is_in_row(*digit))
            .filter(move |digit| !is_in_col(*digit))
            .filter(move |digit| !is_in_group(*digit))
    }
}

impl Default for Sudoku {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy)]
pub struct WriteDigit {
    index: u8,
    digit: u8,
}

impl Problem for Sudoku {
    type Posibility = WriteDigit;
    type Solution = Sudoku;

    // We look over all posibilities for the first free index
    fn extend_possibilities(&self, possible_moves: &mut Vec<WriteDigit>, _history: &[WriteDigit]) {
        // We only consider fields which do not have a digit written into them yet.
        let free_fields = self.fields.iter().enumerate().filter_map(|(index, digit)| {
            if *digit == 0 {
                Some(index as u8)
            } else {
                None
            }
        });
        // We look for the field with the fewest possible digits, and return all its possibilities.
        // Therfore we keep track of the current minimum.
        let mut min = None;
        for index in free_fields {
            let min_count = min.map(|(_, count)| count).unwrap_or(9);
            let new_count = self
                .possible_digits_at(index)
                // We are only interessted in the new count, if it is less than the current minimum,
                // so we can short circut, in case we already have more elements found
                .take(min_count)
                .count();
            if new_count == 0 {
                // Not even one possible digit could be found for this field. This implies that this
                // Sudoku is unsolvable and has no possible moves, since we verified that this field
                // is free. => We short circut, leaving possible_moves empty
                return;
            }
            if new_count < min_count {
                // We found a new minimum, let's remember its index
                min = Some((index, new_count))
            }
        }
        if let Some((index, _count)) = min {
            possible_moves.extend(
                self.possible_digits_at(index)
                    .map(|digit| WriteDigit { index, digit }),
            );
        }
    }

    fn undo(&mut self, last: &WriteDigit, _history: &[WriteDigit]) {
        self.fields[last.index as usize] = 0;
    }

    fn what_if(&mut self, move_: WriteDigit) {
        self.fields[move_.index as usize] = move_.digit;
    }

    fn is_solution(&self, _history: &[WriteDigit]) -> Option<Self::Solution> {
        if self.fields.iter().all(|digit| *digit != 0) {
            Some(self.clone())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::lib::Game;

    use super::{Sudoku, WriteDigit};

    #[test]
    fn print_empty_sudoku() {
        let mut out = Vec::new();
        let game = Sudoku::new();

        game.print_to(&mut out).unwrap();

        let expect = "XXXXXXXXX\n\
            XXXXXXXXX\n\
            XXXXXXXXX\n\
            XXXXXXXXX\n\
            XXXXXXXXX\n\
            XXXXXXXXX\n\
            XXXXXXXXX\n\
            XXXXXXXXX\n\
            XXXXXXXXX\n\
        ";
        assert_eq!(expect, std::str::from_utf8(&out).unwrap());
    }

    #[test]
    fn print_with_first_row_filled() {
        let mut out = Vec::new();
        let mut game = Sudoku::new();
        game.play_move(WriteDigit { index: 0, digit: 1 });
        game.play_move(WriteDigit { index: 1, digit: 2 });
        game.play_move(WriteDigit { index: 2, digit: 3 });
        game.play_move(WriteDigit { index: 3, digit: 4 });
        game.play_move(WriteDigit { index: 4, digit: 5 });
        game.play_move(WriteDigit { index: 5, digit: 6 });
        game.play_move(WriteDigit { index: 6, digit: 7 });
        game.play_move(WriteDigit { index: 7, digit: 8 });
        game.play_move(WriteDigit { index: 8, digit: 9 });

        game.print_to(&mut out).unwrap();

        let expect = "123456789\n\
            XXXXXXXXX\n\
            XXXXXXXXX\n\
            XXXXXXXXX\n\
            XXXXXXXXX\n\
            XXXXXXXXX\n\
            XXXXXXXXX\n\
            XXXXXXXXX\n\
            XXXXXXXXX\n\
        ";
        assert_eq!(expect, std::str::from_utf8(&out).unwrap());
    }

    #[test]
    fn prevent_same_digit_twice_in_same_row() {
        let mut game = Sudoku::new();
        game.play_move(WriteDigit { index: 0, digit: 2 });
        game.play_move(WriteDigit { index: 8, digit: 5 });
        // Won't play a role, because neither same group, row or column
        game.play_move(WriteDigit {
            index: 7 * 9 + 6,
            digit: 5,
        });

        let possibilities = game.possible_digits_at(1).collect::<Vec<u8>>();

        assert_eq!(&[1u8, 3, 4, 6, 7, 8, 9][..], possibilities);
    }

    #[test]
    fn prevent_same_digit_twice_in_same_col() {
        let mut game = Sudoku::new();
        game.play_move(WriteDigit { index: 3, digit: 2 });
        game.play_move(WriteDigit {
            index: 3 + 9 * 5,
            digit: 5,
        });

        let possibilities = game.possible_digits_at(3 + 9 * 2).collect::<Vec<u8>>();

        assert_eq!(&[1u8, 3, 4, 6, 7, 8, 9][..], possibilities);
    }

    #[test]
    fn short_ciruct_if_one_field_has_no_more_possibile_digits() {
        let mut game = Sudoku::new();
        game.play_move(WriteDigit { index: 0, digit: 1 });
        game.play_move(WriteDigit { index: 1, digit: 2 });
        game.play_move(WriteDigit { index: 2, digit: 3 });
        game.play_move(WriteDigit { index: 3, digit: 4 });
        game.play_move(WriteDigit { index: 4, digit: 5 });
        game.play_move(WriteDigit { index: 5, digit: 6 });
        game.play_move(WriteDigit { index: 6, digit: 7 });
        game.play_move(WriteDigit { index: 7, digit: 8 });
        game.play_move(WriteDigit {
            index: 9 + 8,
            digit: 9,
        });

        let mut possible_moves = Vec::new();
        game.fill_possible_moves(&mut possible_moves);

        assert_eq!(0, game.possible_digits_at(8).count());
        assert!(possible_moves.is_empty());
    }
}
