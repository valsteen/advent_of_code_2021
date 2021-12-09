use std::io;
use std::io::BufRead;

struct Map {
    grid: Vec<Vec<u8>>,
    width: usize,
    height: usize,
}

impl Map {
    fn new(grid: Vec<Vec<u8>>) -> Self {
        Self { width: grid.get(0).unwrap().len(), height: grid.len(), grid }
    }

    fn at(&self, x: usize, y: usize) -> u8 {
        *self.grid.get(y).unwrap().get(x).unwrap()
    }

    fn neighbours(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        let mut result = vec![];
        if x > 0 {
            result.push((x - 1, y))
        }
        if x < self.width - 1 {
            result.push((x + 1, y))
        }
        if y > 0 {
            result.push((x, y - 1))
        }
        if y < self.height - 1 {
            result.push((x, y + 1))
        }
        result
    }

    fn is_lowpoint(&self, x: usize, y: usize) -> bool {
        let current = self.at(x, y);
        for (x, y) in self.neighbours(x, y) {
            if current >= self.at(x, y) {
                return false;
            }
        }
        true
    }
}

fn main() {
    let stdin = io::stdin();
    let lines: Vec<Vec<u8>> = stdin
        .lock()
        .lines()
        .flatten()
        .map(|line| line.bytes().map(|x| x - b'0').collect())
        .collect();
    let map = Map::new(lines);

    let mut risks: usize = 0;
    for x in 0..map.width {
        for y in 0..map.height {
            if map.is_lowpoint(x, y) {
                risks += map.at(x, y) as usize + 1
            }
        }
    }
    println!("{:?}", risks);
}
