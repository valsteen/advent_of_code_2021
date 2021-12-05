use itertools::Itertools;
use std::collections::HashMap;
use std::io;
use std::io::Read;
use std::iter::successors;

fn main() {
    let mut grid = HashMap::<_, usize>::new();

    let stdin = io::stdin();
    let lines = stdin.lock().bytes().flatten().map_into::<char>().group_by(|c| c.is_digit(10));
    let lines = lines.into_iter().map(|(_, v)| String::from_iter(v).parse()).flatten().tuples();

    for (x1, y1, x2, y2) in lines {
        let dx: i32 = x2 - x1;
        let dy: i32 = y2 - y1;

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
