use std::cmp::Ordering;
use std::io;
use std::io::{BufRead, Stdin};

fn compute(mut lines: Vec<Vec<u8>>, default: u8) -> Option<usize> {
    let width = lines.get(0).unwrap().len();
    for x in 0..width {
        let ones: usize = (0..lines.len())
            .map(|y| *lines.get(y).unwrap().get(x).unwrap() as usize)
            .sum();
        let keep = match (ones * 2).cmp(&lines.len()) {
            Ordering::Less => (default + 1) % 2,
            Ordering::Greater | Ordering::Equal => default,
        };

        lines = lines
            .into_iter()
            .filter(|line| *line.get(x).unwrap() == keep)
            .collect();

        if lines.len() == 1 {
            let line = lines.pop().unwrap();
            return Some(line.iter().fold(0usize, |acc, x| acc * 2 + *x as usize));
        }
    }
    None
}

fn main() {
    let lines: Vec<Vec<u8>> = Reader::new().lines().collect();
    let oxygen = compute(lines.clone(), 1).unwrap();
    let co2 = compute(lines, 0).unwrap();
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
