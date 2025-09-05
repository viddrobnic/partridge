pub mod solver;

pub const SIZE: usize = 9;
pub const BOARD_SIZE: usize = (SIZE * (SIZE + 1)) / 2;

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct Row(u64);

#[inline]
pub fn card_bits(card: u8) -> u64 {
    debug_assert!(card > 0);
    (1 << card) - 1
}

impl Row {
    #[inline]
    pub fn can_place(&self, card_bits: u64, offset: u8) -> bool {
        (card_bits << offset) & self.0 == 0
    }

    #[inline]
    pub fn place(&mut self, card_bits: u64, offset: u8) {
        self.0 |= card_bits << offset;
    }

    #[inline]
    pub fn remove(&mut self, card_bits: u64, offset: u8) {
        self.0 &= !(card_bits << offset);
    }

    #[inline]
    pub fn is_empty(&self, offset: u8) -> bool {
        self.0 & (1 << offset) == 0
    }
}

#[derive(Clone)]
pub struct Board([Row; BOARD_SIZE]);

impl Default for Board {
    fn default() -> Self {
        Self([Row::default(); BOARD_SIZE])
    }
}

impl Board {
    pub fn can_place(&self, card: u8, x: u8, y: u8) -> bool {
        if (x + card - 1) as usize >= BOARD_SIZE || (y + card - 1) as usize >= BOARD_SIZE {
            return false;
        }

        let c_bits = card_bits(card);

        for dy in 0..card {
            let can_place = self.0[(y + dy) as usize].can_place(c_bits, x);
            if !can_place {
                return false;
            }
        }

        true
    }

    pub fn place(&mut self, card: u8, x: u8, y: u8) {
        let c_bits = card_bits(card);

        for dy in 0..card {
            self.0[(y + dy) as usize].place(c_bits, x);
        }
    }

    pub fn remove(&mut self, card: u8, x: u8, y: u8) {
        let c_bits = card_bits(card);

        for dy in 0..card {
            self.0[(y + dy) as usize].remove(c_bits, x);
        }
    }

    pub fn find_empty(&self, y: u8) -> Option<(u8, u8)> {
        for y in y..(BOARD_SIZE as u8) {
            let row = &self.0[y as usize];
            for x in 0..BOARD_SIZE {
                if row.is_empty(x as u8) {
                    return Some((x as u8, y));
                }
            }
        }

        None
    }
}

pub type Solution = [u8; BOARD_SIZE];

#[cfg(test)]
mod test {
    use crate::{Board, Row, SIZE};

    #[test]
    fn card_bits() {
        assert_eq!(super::card_bits(1), 0b1);
        assert_eq!(super::card_bits(2), 0b11);
        assert_eq!(super::card_bits(3), 0b111);
        assert_eq!(super::card_bits(4), 0b1111);
        assert_eq!(super::card_bits(5), 0b11111);
        assert_eq!(super::card_bits(6), 0b111111);
        assert_eq!(super::card_bits(7), 0b1111111);
        assert_eq!(super::card_bits(8), 0b11111111);
        assert_eq!(super::card_bits(9), 0b111111111);
    }

    #[test]
    fn row_idenpotent() {
        let mut row = Row::default();
        for card in 1..=SIZE {
            let card_bits = super::card_bits(card as u8);
            let can = row.can_place(card_bits, 0);
            assert!(can);

            row.place(card_bits, 0);
            row.remove(card_bits, 0);
            assert_eq!(row.0, 0);
        }
    }

    #[test]
    fn board_idenpotent() {
        let mut board = Board::default();
        for card in 1..=SIZE {
            let can = board.can_place(card as u8, 0, 0);
            assert!(can);

            board.place(card as u8, 0, 0);
            board.remove(card as u8, 0, 0);

            let sum: u64 = board.0.iter().map(|row| row.0).sum();
            assert_eq!(sum, 0);
        }
    }
}
