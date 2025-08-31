const SIZE: usize = 9;
const BOARD_SIZE: usize = (SIZE * (SIZE + 1)) / 2;

#[derive(Default, Clone, Copy, PartialEq, Eq)]
struct Row(u64);

#[inline]
fn card_bits(card: u8) -> u64 {
    debug_assert!(card > 0);
    (1 << card) - 1
}

impl Row {
    #[inline]
    fn can_place(&self, card_bits: u64, offset: u8) -> bool {
        (card_bits << offset) & self.0 == 0
    }

    #[inline]
    fn place(&mut self, card_bits: u64, offset: u8) {
        self.0 |= card_bits << offset;
    }

    #[inline]
    fn remove(&mut self, card_bits: u64, offset: u8) {
        self.0 &= !(card_bits << offset);
    }

    #[inline]
    fn is_empty(&self, offset: u8) -> bool {
        self.0 & (1 << offset) == 0
    }
}

struct Board([Row; BOARD_SIZE]);

impl Board {
    fn new() -> Self {
        Self([Row::default(); BOARD_SIZE])
    }

    fn can_place(&self, card: u8, x: u8, y: u8) -> bool {
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

    fn place(&mut self, card: u8, x: u8, y: u8) {
        let c_bits = card_bits(card);

        for dy in 0..card {
            self.0[(y + dy) as usize].place(c_bits, x);
        }
    }

    fn remove(&mut self, card: u8, x: u8, y: u8) {
        let c_bits = card_bits(card);

        for dy in 0..card {
            self.0[(y + dy) as usize].remove(c_bits, x);
        }
    }

    fn find_empty(&self, y: u8) -> Option<(u8, u8)> {
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

struct Solution {
    steps: [u8; BOARD_SIZE],
    length: usize,
}

impl Default for Solution {
    fn default() -> Self {
        Self {
            steps: [0; BOARD_SIZE],
            length: 0,
        }
    }
}

impl Solution {
    fn push(&mut self, card: u8) {
        self.steps[self.length] = card;
        self.length += 1;
    }

    fn pop(&mut self) {
        self.length -= 1;
    }
}

fn solve(
    board: &mut Board,
    free_cards: &mut [u8; SIZE],
    solution: &mut Solution,
    nr_solutions: &mut u32,
    x: u8,
    y: u8,
) {
    let mut increased = false;
    for card_idx in 0..SIZE {
        if free_cards[card_idx] == 0 {
            continue;
        }

        let card = (card_idx + 1) as u8;
        if !board.can_place(card, x, y) {
            continue;
        }

        solution.push(card);
        free_cards[card_idx] -= 1;
        board.place(card, x, y);

        match board.find_empty(y) {
            Some((new_x, new_y)) => solve(board, free_cards, solution, nr_solutions, new_x, new_y),
            None => {
                *nr_solutions += 1;
                increased = true;
            }
        };

        solution.pop();
        free_cards[card_idx] += 1;
        board.remove(card, x, y);
    }

    if increased && *nr_solutions % 10 == 0 {
        println!("{nr_solutions}");
    }
}

fn main() {
    let mut free_cards = [0u8; SIZE];
    for (card_idx, left) in free_cards.iter_mut().enumerate() {
        *left = card_idx as u8 + 1;
    }

    let mut board = Board::new();
    let mut solution = Solution::default();
    let mut nr_solutions = 0;
    solve(
        &mut board,
        &mut free_cards,
        &mut solution,
        &mut nr_solutions,
        0,
        0,
    );

    println!("finished: {nr_solutions}");
}

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
        let mut board = Board::new();
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
