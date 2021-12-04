use itertools::Itertools;
use std::io::BufRead;
use std::{io, mem};

fn main() {
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines().flatten();

    let draws: Vec<u8> = lines
        .next()
        .unwrap()
        .split(',')
        .map(str::parse)
        .flatten()
        .collect();

    let lines = lines.skip(1).chain(vec!["".to_string()]);

    let grids: Vec<Vec<Vec<u8>>> = lines
        .scan(vec![], |grid, line| {
            let numbers = line
                .split_whitespace()
                .map(str::parse)
                .flatten()
                .collect_vec();
            if numbers.is_empty() {
                Some(Some(mem::take(grid)))
            } else {
                grid.push(numbers);
                Some(None)
            }
        })
        .flatten()
        .collect();

    for grid in grids {
        println!("{:?}", grid)
    }

    println!("{:?}", draws);
}
