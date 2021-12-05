use itertools::Itertools;
use std::collections::HashMap;
use std::io::Read;
use std::io;

fn main() {
    let mut grid = HashMap::<_, usize>::new();

    let stdin = io::stdin();
    let lines = stdin.lock().bytes().flatten().map_into::<char>().group_by(|c| c.is_digit(10));
    let lines =
        lines.into_iter().map(|(_, v)| String::from_iter(v).parse::<i32>()).flatten().tuples();

    for (x1, y1, x2, y2) in lines {
        if x1 == x2 {
            for y in [y1..=y2, y2..=y1].into_iter().flatten() {
                *grid.entry((x1, y)).or_default() += 1;
            }
        }
        if y1 == y2 {
            for x in [x1..=x2, x2..=x1].into_iter().flatten() {
                *grid.entry((x, y1)).or_default() += 1;
            }
        }
    }

    println!("{}", grid.values().filter(|v| **v > 1).count())
}
