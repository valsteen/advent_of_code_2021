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
        .tuples::<(usize, usize, usize, usize)>();

    let mut grid = HashMap::<(usize, usize), usize>::new();
    for line in lines {
        if line.0 == line.2 {
            for y in [line.1..=line.3, line.3..=line.1].into_iter().flatten() {
                *grid.entry((line.0, y)).or_default() += 1;
            }
        }
        if line.1 == line.3 {
            for x in [line.0..=line.2, line.2..=line.0].into_iter().flatten() {
                *grid.entry((x, line.1)).or_default() += 1;
            }
        }
    }

    println!("{}", grid.values().filter(|v| **v > 1).count())
}
