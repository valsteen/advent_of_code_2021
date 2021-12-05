use itertools::Itertools;
use std::collections::HashMap;
use std::io::Read;
use std::iter::successors;
use std::{io, mem};

fn main() {
    let stdin = io::stdin();
    let lines = stdin
        .lock()
        .bytes()
        .flatten()
        .map_into()
        .scan(String::new(), |acc, d| {
            if ('0'..='9').contains(&d) {
                acc.push(d);
                Some(None)
            } else {
                Some(Some(mem::take(acc)))
            }
        })
        .flatten()
        .map(|s| s.parse::<i32>())
        .flatten()
        .tuples();

    let mut grid = HashMap::<_, usize>::new();
    for (x1, y1, x2, y2) in lines {
        let dx = x2 - x1;
        let dy = y2 - y1;

        let incx = if dx == 0 { 0 } else { dx / dx.abs() };
        let incy = if dy == 0 { 0 } else { dy / dy.abs() };

        for (x, y) in successors(Some((x1, y1)), |(x, y)| {
            ((*x, *y) != (x2, y2)).then(|| (x + incx, y + incy))
        }) {
            *grid.entry((x, y)).or_default() += 1;
        }
    }

    println!("{}", grid.values().filter(|v| **v > 1).count())
}
