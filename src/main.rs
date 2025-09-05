use std::fs;
use std::io::{BufWriter, Write};
use std::thread;

use partridge::solver::Solver;

fn main() {
    solve();
}

fn solve() {
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
