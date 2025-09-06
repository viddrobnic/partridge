use std::{
    fs,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use image::{
    Rgb, RgbImage,
    codecs::png::{CompressionType, FilterType, PngEncoder},
};

use crate::{BOARD_SIZE, Board, Solution};

const SQUARE_SIZE: usize = 10;

const COLORS: [Rgb<u8>; 9] = [
    Rgb([0xef, 0x44, 0x44]),
    Rgb([0xea, 0xb3, 0x08]),
    Rgb([0x06, 0xb6, 0xd4]),
    Rgb([0x8b, 0x5c, 0xf6]),
    Rgb([0x22, 0xc5, 0x5e]),
    Rgb([0xd9, 0x46, 0xef]),
    Rgb([0x3b, 0x82, 0xf6]),
    Rgb([0xf4, 0x3f, 0x5e]),
    Rgb([0xf9, 0x73, 0x16]),
];

pub fn render(solutions: &[Solution], start: usize, counter: Arc<AtomicUsize>) {
    for (idx, sol) in solutions.iter().enumerate() {
        let img = render_solution(sol);

        let mut f = fs::File::create(format!("images/{:07}.png", start + idx)).unwrap();
        let encoder =
            PngEncoder::new_with_quality(&mut f, CompressionType::Best, FilterType::Adaptive);

        img.write_with_encoder(encoder).unwrap();

        let count = counter.fetch_add(1, Ordering::Relaxed);
        if count % 1000 == 0 {
            println!("progress: {count}");
        }
    }
}

fn render_solution(sol: &Solution) -> RgbImage {
    let size = (BOARD_SIZE * SQUARE_SIZE) as u32;
    let mut img = RgbImage::new(size, size);
    let mut board = Board::default();
    let mut x = 0;
    let mut y = 0;

    for (idx, card) in sol.iter().enumerate() {
        let start_y = y * SQUARE_SIZE;
        let end_y = (y + *card as usize) * SQUARE_SIZE;
        let start_x = x * SQUARE_SIZE;
        let end_x = (x + *card as usize) * SQUARE_SIZE;

        let color = COLORS[*card as usize - 1];
        for y in start_y..end_y {
            for x in start_x..end_x {
                if y == start_y || y == end_y || x == start_x || x == end_x {
                    img.put_pixel(x as u32, y as u32, Rgb([0, 0, 0]));
                } else {
                    img.put_pixel(x as u32, y as u32, color);
                }
            }
        }

        if idx < sol.len() - 1 {
            board.place(*card, x as u8, y as u8);
            let (new_x, new_y) = board.find_empty(y as u8).unwrap();
            x = new_x as usize;
            y = new_y as usize;
        }
    }

    img
}
