use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
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

struct Visit {
    x: i32,
    y: i32,
    score: (i32, i32, usize),
}

impl Eq for Visit {}

impl PartialEq<Self> for Visit {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl PartialOrd<Self> for Visit {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Visit {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score.2.cmp(&other.score.2).reverse().then(
            self.score.0.cmp(&other.score.0)
        )
    }
}

fn visit(map: &HashMap<(i32, i32), u8>, width: i32, height: i32) -> usize {
    let mut best = usize::MAX;
    let mut to_visit = BinaryHeap::new();
    let mut scores = HashMap::<(i32, i32), usize>::from_iter(map.iter().map(|(&(x,y), _)|{
        ((x,y), usize::MAX)
    }));

    scores.insert((0, 0), 0);
    to_visit.push(Visit { x: 0, y: 0, score: (0, 0, 0) });

    while !to_visit.is_empty() {
        let Visit { x: start_x, y: start_y, .. } = to_visit.pop().unwrap();

        let &score = scores.get(&(start_x, start_y)).unwrap();
        neighbours(start_x, start_y, width, height, |x, y| {
            let &risk = map.get(&(x, y)).unwrap();

            let current = risk as usize + score;
            let previous_best = scores.get_mut(&(x, y)).unwrap();

            if *previous_best <= current || best <= current {
                return;
            }

            *previous_best = current ;

            if x == width - 1 && y == height - 1 {
                best = best.min(current);
                return;
            }

            to_visit.push(Visit { x, y, score: (x + y, i32::abs(x - y), current) });
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
