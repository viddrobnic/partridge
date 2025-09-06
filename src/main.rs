use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::thread;

use clap::{Parser, Subcommand};
use partridge::solver::Solver;
use partridge::{render, solution_from_str};

const FILE_NAME: &str = "solutions.txt";

#[derive(Parser)]
#[command(version, about, long_about = None)]
/// Solve and render the partridge puzzle.
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Solve the puzzle
    Solve,
    /// Render the puzzle
    Render,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Solve => solve(),
        Command::Render => render_all(),
    }
}

fn solve() {
    let threads = thread::available_parallelism().unwrap().get() as u8;
    println!("running on {threads} threads!");

    let mut solver = Solver::new(threads);
    solver.solve(0, 0);

    let solutions = solver.solutions.lock().unwrap();
    let f = fs::File::create(FILE_NAME).unwrap();
    let mut w = BufWriter::new(f);

    for sol in solutions.iter() {
        for c in sol {
            _ = write!(&mut w, "{} ", c);
        }
        _ = writeln!(&mut w);
    }
}

fn render_all() {
    let f = fs::File::open(FILE_NAME).unwrap();
    let r = BufReader::new(f);

    let solutions: Vec<_> = r
        .lines()
        .map(|line| solution_from_str(&line.unwrap()))
        .collect();

    let threads = thread::available_parallelism().unwrap().get();
    println!("running on {threads} threads!");

    let counter = Arc::new(AtomicUsize::new(0));

    let mut handles = vec![];
    let per_thread = solutions.len() / threads;
    for i in 0..threads {
        let start = per_thread * i;
        let end = if i == threads - 1 {
            solutions.len()
        } else {
            per_thread * (i + 1)
        };
        let sols = solutions[start..end].to_vec();

        let c = counter.clone();
        let handle = thread::spawn(move || render::render(&sols, start, c));
        handles.push(handle);
    }

    for h in handles {
        h.join().unwrap();
    }
}
