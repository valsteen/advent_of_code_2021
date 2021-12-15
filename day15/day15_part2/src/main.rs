use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};
use std::io;
use std::io::BufRead;

fn neighbours(x: usize, y: usize, width: usize, height: usize, mut f: impl FnMut(usize, usize)) {
    if x > 0 {
        f(x - 1, y);
    }

    if y > 0 {
        f(x, y - 1)
    }

    if y < height - 1 {
        f(x, y + 1)
    }

    if x < width - 1 {
        f(x + 1, y);
    }
}

fn visit(map: &[Vec<u8>]) -> usize {
    let height = map.len();
    let width = map.get(0).unwrap().len();

    let mut best = usize::MAX;
    let mut to_visit = HashSet::new();
    let mut visited = HashMap::<(usize, usize), usize>::new();
    visited.insert((0, 0), 0);
    to_visit.insert((0, 0));

    while !to_visit.is_empty() {
        let &(start_x, start_y) = to_visit
            .iter()
            .max_by_key(|&&(x, y)| (Reverse(x + y), *visited.entry((x, y)).or_insert(usize::MAX)))
            .unwrap();

        to_visit.remove(&(start_x, start_y));

        let &mut score = visited.entry((start_x, start_y)).or_insert(usize::MAX);
        neighbours(start_x, start_y, width * 5, height * 5, |x, y| {
            let row = map.get(y % height).unwrap();
            let &risk = row.get(x % width).unwrap();
            let risk = (risk - 1 + (x / width) as u8 + (y / height) as u8) % 9 + 1;

            let current = risk as usize + score;
            let &mut previous_best = visited.entry((x, y)).or_insert(usize::MAX);

            if previous_best <= current || best <= current {
                return;
            }

            visited.insert((x, y), current);

            if x == width * 5 - 1 && y == height * 5 - 1 {
                best = best.min(current);
                return;
            }

            to_visit.insert((x, y));
        });
    }
    best
}

fn main() {
    let stdin = io::stdin();
    let risks: Vec<Vec<u8>> = stdin
        .lock()
        .lines()
        .flatten()
        .map(|line| line.bytes().map(|x| x - b'0').collect())
        .collect();

    let best = visit(&risks);
    println!("{}", best);
}
