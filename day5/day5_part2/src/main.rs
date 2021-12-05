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
        .map(|s| s.parse::<i32>())
        .flatten()
        .tuples();

    let mut grid = HashMap::<_, usize>::new();
    for (x1, y1, x2, y2) in lines {
        let dx = x2 - x1;
        let dy = y2 - y1;

        let mut pos = (x1, y1);
        let incx = if dx == 0 { 0 } else { dx / dx.abs() };
        let incy = if dy == 0 { 0 } else { dy / dy.abs() };

        loop {
            *grid.entry(pos).or_default() += 1;
            if pos == (x2, y2) {
                break;
            }
            pos.0 += incx;
            pos.1 += incy;
        }
    }

    println!("{}", grid.values().filter(|v| **v > 1).count())
}
