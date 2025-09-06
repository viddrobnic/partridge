use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::*;

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
pub struct Solver {
    board: Board,
    free_cards: [u8; SIZE],
    solution: SolutionBuilder,

    depth: u8,
    max_threads: u8,
    nr_threads: Arc<AtomicU8>,

    pub solutions: Arc<Mutex<Vec<Solution>>>,
}

impl Solver {
    pub fn new(max_threads: u8) -> Self {
        let mut free_cards = [0u8; SIZE];
        for (card_idx, left) in free_cards.iter_mut().enumerate() {
            *left = card_idx as u8 + 1;
        }

        Self {
            free_cards,
            board: Board::default(),
            solution: SolutionBuilder::default(),
            depth: 0,
            max_threads,
            nr_threads: Arc::new(AtomicU8::new(1)),
            solutions: Default::default(),
        }
    }

    pub fn solve(&mut self, x: u8, y: u8) {
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
        self.depth < 10 && self.nr_threads.load(Ordering::Relaxed) < self.max_threads
    }
}
