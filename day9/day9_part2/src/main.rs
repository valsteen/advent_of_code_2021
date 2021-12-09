use std::collections::HashMap;
use std::io;
use std::io::BufRead;

struct Map {
    grid: Vec<Vec<u8>>,
    width: usize,
    height: usize,
    bassins: HashMap<(usize, usize), usize>,
}

impl Map {
    fn new(grid: Vec<Vec<u8>>) -> Self {
        Self {
            width: grid.get(0).unwrap().len(),
            height: grid.len(),
            bassins: HashMap::new(),
            grid,
        }
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

    fn mark_bassin(&mut self, x: usize, y: usize, id: usize) {
        self.bassins.insert((x, y), id);
        for (x1, y1) in self.neighbours(x, y) {
            if !self.bassins.contains_key(&(x1, y1))
                && (self.at(x, y)..9).contains(&self.at(x1, y1))
            {
                self.mark_bassin(x1, y1, id);
            }
        }
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
    let mut map = Map::new(lines);

    let mut id = 0;
    for x in 0..map.width {
        for y in 0..map.height {
            if map.is_lowpoint(x, y) {
                map.mark_bassin(x, y, id);
                id += 1;
            }
        }
    }

    let mut sizes = vec![0; id];

    for (_, id) in map.bassins {
        sizes[id] += 1;
    }

    sizes.sort_unstable();

    println!("{}", sizes[sizes.len() - 3..sizes.len()].iter().product::<usize>());
}
