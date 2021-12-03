use std::cmp::Ordering;
use std::io;
use std::io::{BufRead, Stdin};

fn compute(lines: Vec<Vec<u8>>, x: usize, default: u8) -> usize {
    if lines.len() == 1 {
        let line = lines.first().unwrap();
        return line.iter().fold(0usize, |acc, x| acc * 2 + *x as usize);
    }

    let ones: usize = lines
        .iter()
        .map(|line| *line.get(x).unwrap() as usize)
        .sum();
    let keep = match (ones * 2).cmp(&lines.len()) {
        Ordering::Less => (default + 1) % 2,
        Ordering::Greater | Ordering::Equal => default,
    };

    compute(
        lines
            .into_iter()
            .filter(|line| *line.get(x).unwrap() == keep)
            .collect(),
        x + 1,
        default,
    )
}

fn main() {
    let lines: Vec<Vec<u8>> = Reader::new().lines().collect();
    let oxygen = compute(lines.clone(), 0, 1);
    let co2 = compute(lines, 0, 0);
    println!("{} {} {}", oxygen, co2, oxygen * co2)
}

struct Reader {
    stdin: Stdin,
}

impl Reader {
    fn new() -> Self {
        Self { stdin: io::stdin() }
    }

    fn lines(&mut self) -> impl Iterator<Item = Vec<u8>> + '_ {
        self.stdin.lock().lines().flatten().map(|s| {
            s.chars()
                .map(|c| match c {
                    '0' => 0,
                    '1' => 1,
                    _ => panic!("invalid input {}", c),
                })
                .collect()
        })
    }
}
