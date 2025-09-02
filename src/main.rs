use std::{
    fs,
    io::{BufWriter, Write},
    sync::{
        Arc, Mutex,
        atomic::{AtomicU8, Ordering},
    },
    thread,
};

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

#[derive(Clone)]
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

type Solution = [u8; BOARD_SIZE];

#[derive(Clone)]
struct SolutionBuilder {
    steps: Solution,
    length: usize,
}

impl Default for SolutionBuilder {
    fn default() -> Self {
        Self {
            steps: [0; BOARD_SIZE],
            length: 0,
        }
    }
}

impl SolutionBuilder {
    fn push(&mut self, card: u8) {
        self.steps[self.length] = card;
        self.length += 1;
    }

    fn pop(&mut self) {
        self.length -= 1;
    }
}

#[derive(Clone)]
struct Solver {
    board: Board,
    free_cards: [u8; SIZE],
    solution: SolutionBuilder,

    depth: u8,
    max_threads: u8,
    nr_threads: Arc<AtomicU8>,

    solutions: Arc<Mutex<Vec<Solution>>>,
}

impl Solver {
    fn new(max_threads: u8) -> Self {
        let mut free_cards = [0u8; SIZE];
        for (card_idx, left) in free_cards.iter_mut().enumerate() {
            *left = card_idx as u8 + 1;
        }

        Self {
            free_cards,
            board: Board::new(),
            solution: SolutionBuilder::default(),
            depth: 0,
            max_threads,
            nr_threads: Arc::new(AtomicU8::new(1)),
            solutions: Default::default(),
        }
    }

    fn solve(&mut self, x: u8, y: u8) {
        for card_idx in 0..SIZE {
            if self.free_cards[card_idx] == 0 {
                continue;
            }

            let card = (card_idx + 1) as u8;
            if !self.board.can_place(card, x, y) {
                continue;
            }

            self.solution.push(card);
            self.free_cards[card_idx] -= 1;
            self.board.place(card, x, y);
            self.depth += 1;

            match self.board.find_empty(y) {
                None => {
                    self.add_solution(self.solution.steps);
                }
                Some((new_x, new_y)) => {
                    if self.should_parallelise() {
                        self.nr_threads.fetch_add(1, Ordering::Relaxed);
                        let mut solver = self.clone();

                        thread::spawn(move || {
                            solver.solve(new_x, new_y);
                            solver.nr_threads.fetch_sub(1, Ordering::Relaxed);
                        });
                    } else {
                        self.solve(new_x, new_y);
                    }
                }
            };

            self.solution.pop();
            self.free_cards[card_idx] += 1;
            self.board.remove(card, x, y);
            self.depth -= 1;
        }
    }

    fn add_solution(&self, solution: Solution) {
        let mut sols = self.solutions.lock().unwrap();
        sols.push(solution);

        if sols.len() % 10 == 0 {
            println!("progress: {}", sols.len());
        }
    }

    #[inline]
    fn should_parallelise(&self) -> bool {
        self.depth < 20 && self.nr_threads.load(Ordering::Relaxed) < self.max_threads
    }
}

fn main() {
    let threads = thread::available_parallelism().unwrap().get() as u8;
    println!("running on {threads} threads!");

    let mut solver = Solver::new(threads);
    solver.solve(0, 0);

    let solutions = solver.solutions.lock().unwrap();
    let f = fs::File::create("solutions.txt").unwrap();
    let mut w = BufWriter::new(f);

    for sol in solutions.iter() {
        for c in sol {
            _ = write!(&mut w, "{} ", c);
        }
        _ = writeln!(&mut w);
    }
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
