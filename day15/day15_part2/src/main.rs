use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};
use std::io;
use std::io::BufRead;

fn neighbours(x: i32, y: i32, width: i32, height: i32, mut f: impl FnMut(i32, i32)) {
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

fn visit(map: &HashMap<(i32, i32), u8>, width: i32, height: i32) -> usize {
    let mut best = usize::MAX;
    let mut to_visit = HashSet::new();
    let mut scores = HashMap::<(i32, i32), usize>::new();
    scores.insert((0, 0), 0);
    to_visit.insert((0, 0));

    while !to_visit.is_empty() {
        let &(start_x, start_y) = to_visit
            .iter()
            .max_by_key(|&&(x, y)| {
                //, Reverse(*scores.entry((x, y)).or_insert(usize::MAX)))
                (
                    Reverse(x + y),
                    Reverse(i32::abs(x - y)),
                    Reverse(*scores.entry((x, y)).or_insert(usize::MAX)),
                )
            })
            .unwrap();

        to_visit.remove(&(start_x, start_y));

        let &mut score = scores.entry((start_x, start_y)).or_insert(usize::MAX);
        neighbours(start_x, start_y, width, height, |x, y| {
            let &risk = map.get(&(x, y)).unwrap();

            let current = risk as usize + score;
            let &mut previous_best = scores.entry((x, y)).or_insert(usize::MAX);

            if previous_best <= current || best <= current {
                return;
            }

            scores.insert((x, y), current);

            if x == width - 1 && y == height - 1 {
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
    let mut max_x: i32 = 0;
    let mut max_y: i32 = 0;

    let mut map: HashMap<(i32, i32), u8> = stdin
        .lock()
        .lines()
        .flatten()
        .enumerate()
        .map(|(y, line)| {
            line.bytes()
                .enumerate()
                .map(|(x, c)| {
                    max_x = max_x.max(x as i32);
                    max_y = max_y.max(y as i32);
                    ((x as i32, y as i32), c - b'0')
                })
                .collect::<Vec<((i32, i32), u8)>>()
        })
        .flatten()
        .collect();

    let height = max_y + 1;
    let width = max_x + 1;

    for y in 0..height * 5 {
        for x in 0..width * 5 {
            let &risk = map.get(&(x % width, y % height)).unwrap();
            let new_risk = (risk - 1 + (x / width) as u8 + (y / height) as u8) % 9 + 1;
            map.insert((x, y), new_risk);
        }
    }

    let best = visit(&map, width * 5, height * 5);

    println!("{}", best);
}
