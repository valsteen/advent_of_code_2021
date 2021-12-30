#![cfg_attr(feature = "pedantic", warn(clippy::pedantic))]
#![warn(clippy::use_self)]
#![warn(clippy::map_flatten)]
#![warn(clippy::map_unwrap_or)]
#![warn(deprecated_in_future)]
#![warn(future_incompatible)]
#![warn(noop_method_call)]
#![warn(missing_debug_implementations)]
#![warn(rust_2018_compatibility)]
#![warn(rust_2021_compatibility)]
#![warn(rust_2018_idioms)]
#![warn(unused)]
//#![deny(warnings)]

use std::collections::HashMap;
use std::io;
use std::io::Read;
use std::ops::ControlFlow;

#[derive(Clone, Copy, Debug)]
enum Tile {
    South,
    East,
    Empty,
}

impl TryFrom<char> for Tile {
    type Error = &'static str;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        let result = match value {
            '.' => Self::Empty,
            'v' => Self::South,
            '>' => Self::East,
            _ => return Err("Invalid tile")
        };
        Ok(result)
    }
}

#[derive(Debug)]
struct Map {
    tiles: HashMap<(usize, usize), Tile>,
    width: usize,
    height: usize
}

impl Map {
    fn east(&self) -> impl Iterator<Item=(usize,usize)> + '_ {
        self.tiles.iter().filter_map(|((x,y),tile)| match tile {
            Tile::East => Some((*x,*y)),
            _ => None
        })
    }

    fn south(&self) -> impl Iterator<Item=(usize,usize)> + '_ {
        self.tiles.iter().filter_map(|((x,y),tile)| match tile {
            Tile::South => Some((*x,*y)),
            _ => None
        })
    }

    fn step(mut self) -> Option<Self> {
        let mut map = Self {
            tiles: Default::default(),
            width: self.width,
            height: self.height
        };
        let mut changed = false;
        let mut moved = vec![];
        for (x,y) in self.east() {
            let next_x = (x + 1) % self.width;
            if self.tiles.get(&(next_x,y)).is_none() {
                moved.push((x,y));
                map.tiles.insert((next_x, y), Tile::East);
                changed = true;
            } else {
                map.tiles.insert((x, y), Tile::East);
            }
        }
        for (x,y) in moved {
            self.tiles.remove(&(x,y));
        }

        for (x,y) in self.south() {
            let next_y = (y + 1) % self.height;
            if self.tiles.get(&(x,next_y)).is_none() && map.tiles.get(&(x,next_y)).is_none() {
                map.tiles.insert((x, next_y), Tile::South);
                changed = true;
            } else {
                map.tiles.insert((x, y), Tile::South);
            }
        };
        if changed {
            Some(map)
        } else {
            None
        }
    }
}

fn main() -> Result<(), &'static str> {
    let stdin = io::stdin();
    let (map, _) = stdin
        .lock()
        .bytes()
        .into_iter()
        .flatten()
        .map(char::from)
        .try_fold((Map{
            tiles: Default::default(),
            width: 0,
            height: 0
        }, (0, 0)), |(mut map, (mut x, mut y)), c| {
            if c == '\n' {
                y += 1;
                x = 0;
                map.height = map.height.max(y);
            } else {
                let tile = Tile::try_from(c)?;
                match tile {
                    Tile::East | Tile::South => { map.tiles.insert ((x, y), tile); },
                    Tile::Empty => {}
                }
                x += 1;
                map.width = map.width.max(x);
            };
            Ok((map, (x,y)))
        })?;

    let max = (1..).into_iter().try_fold(map, |e, i| {
        if let Some(map) = e.step() {
            ControlFlow::Continue(map)
        } else {
            ControlFlow::Break(i)
        }
    });

    if let ControlFlow::Break(i) = max {
        println!("{:?}", i);
    }

    Ok(())
}
