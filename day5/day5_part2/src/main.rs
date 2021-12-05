use itertools::Itertools;
use std::collections::HashMap;
use std::io::Read;
use std::{io, mem};

fn main() {
    let stdin = io::stdin();
    let lines = stdin
        .lock()
        .bytes()
        .flatten()
        .map(char::from)
        .scan(String::new(), |acc, d| {
            if ('0'..='9').contains(&d) {
                acc.push(d);
                Some(None)
            } else {
                Some(Some(mem::take(acc)))
            }
        })
        .flatten()
        .map(|s| s.parse())
        .flatten()
        .tuples::<(i32, i32, i32, i32)>();

    let mut grid = HashMap::<(i32, i32), usize>::new();
    for line in lines {
        let dx = line.2 - line.0;
        let dy = line.3 - line.1;

        let mut pos = (line.0, line.1);
        let incx = if dx == 0 { 0 } else { dx / dx.abs() };
        let incy = if dy == 0 { 0 } else { dy / dy.abs() };

        loop {
            *grid.entry(pos).or_default() += 1;
            if pos == (line.2, line.3) { break }
            pos.0 += incx;
            pos.1 += incy;
        }
    }

    println!("{}", grid.values().filter(|v| **v > 1).count())
}
