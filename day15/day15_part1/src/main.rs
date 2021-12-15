use std::cmp::Ordering;
use std::collections::HashMap;
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

fn visit(
    start_x: usize,
    start_y: usize,
    map: &[Vec<u8>],
    visited: &mut HashMap<(usize, usize), usize>,
    score: usize,
    f: &mut impl FnMut(usize),
) {
    let height = map.len();
    let width = map.get(0).unwrap().len();

    let mut to_visit = vec![];
    neighbours(start_x, start_y, width, height, |x, y| {
        let row = map.get(y).unwrap();
        let &risk = row.get(x).unwrap();
        let current = risk as usize + score;
        let &previous_best = visited.get(&(x, y)).unwrap();

        if previous_best <= current {
            return;
        }

        if x == width - 1 && y == height - 1 {
            f(current);
            return;
        }

        visited.insert((x, y), current);

        to_visit.push((x, y, current));
    });

    to_visit.sort_by(|(x, y, current), (x1, y1, current1)| {
        let cmp = (x1 + y1).cmp(&(x + y));
        match cmp {
            Ordering::Greater | Ordering::Less => cmp,
            Ordering::Equal => current.cmp(current1),
        }
    });

    for (x, y, current) in to_visit {
        let &previous_best = visited.get(&(x, y)).unwrap();
        if current <= previous_best {
            visit(x, y, map, visited, current, f)
        }
    }
}

fn main() {
    let stdin = io::stdin();
    let risks: Vec<Vec<u8>> = stdin
        .lock()
        .lines()
        .flatten()
        .map(|line| line.bytes().map(|x| x - b'0').collect())
        .collect();

    let mut visited = HashMap::new();

    let height = risks.len();
    let width = risks.get(0).unwrap().len();

    for x in 0..width {
        for y in 0..height {
            visited.insert((x, y), usize::MAX);
        }
    }
    visited.insert((0, 0), 0);

    let mut best = usize::MAX;
    visit(0, 0, &risks, &mut visited, 0, &mut |score| {
        if score < best {
            best = score
        }
    });
    println!("{}", best);
}
