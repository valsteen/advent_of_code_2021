use itertools::Itertools;
use std::collections::HashSet;
use std::io::BufRead;
use std::{io, mem};

struct Grid {
    marks: HashSet<(usize, usize)>,
    rows: Vec<Vec<u8>>,
    winner: bool,
}

impl Grid {
    fn new(rows: Vec<Vec<u8>>) -> Self {
        Self {
            marks: HashSet::new(),
            rows,
            winner: false,
        }
    }

    fn winning_column(&self, x: usize) -> bool {
        (0..self.rows.len()).all(|y| self.marks.contains(&(x, y)))
    }

    fn winning_row(&self, y: usize) -> bool {
        (0..self.rows.get(y).unwrap().len()).all(|x| self.marks.contains(&(x, y)))
    }

    fn mark(&mut self, number: u8) {
        for (y, row) in self.rows.iter().enumerate() {
            for x in row
                .iter()
                .enumerate()
                .filter_map(|(x, value)| (*value == number).then(|| x))
            {
                self.marks.insert((x, y));
                self.winner = self.winner || self.winning_column(x) || self.winning_row(y);
            }
        }
    }

    fn score(&self) -> usize {
        self.rows
            .iter()
            .enumerate()
            .map(|(y, row)| {
                row.iter().enumerate().filter_map(move |(x, value)| {
                    (!self.marks.contains(&(x, y))).then(|| *value as usize)
                })
            })
            .flatten()
            .sum()
    }
}

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

    let mut grids: Vec<Grid> = lines
        .scan(vec![], |grid, line| {
            let numbers = line
                .split_whitespace()
                .map(str::parse)
                .flatten()
                .collect_vec();
            if numbers.is_empty() {
                Some(Some(Grid::new(mem::take(grid))))
            } else {
                grid.push(numbers);
                Some(None)
            }
        })
        .flatten()
        .collect();

    'main: for draw in draws {
        for (n, grid) in grids.iter_mut().enumerate() {
            grid.mark(draw);
            if grid.winner {
                println!(
                    "{} wins: {} ; {}",
                    n,
                    grid.score(),
                    draw as usize * grid.score()
                );
                break 'main;
            }
        }
    }
}
