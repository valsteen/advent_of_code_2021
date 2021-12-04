use std::io;
use std::io::{BufRead, Stdin};

fn main() {
    let lines: Vec<Vec<u8>> = Reader::new().lines().collect();
    let width = lines.get(0).unwrap().len();

    let mut gamma = 0;
    let mut epsilon = 0;
    for x in 0..width {
        let ones: usize =
            (0..lines.len()).map(|y| *lines.get(y).unwrap().get(x).unwrap() as usize).sum();
        let one_most_common = ones > lines.len() / 2;
        gamma = gamma * 2 + if one_most_common { 1 } else { 0 };
        epsilon = epsilon * 2 + if one_most_common { 0 } else { 1 };
    }
    println!("{} {} {}", gamma, epsilon, gamma * epsilon);
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
