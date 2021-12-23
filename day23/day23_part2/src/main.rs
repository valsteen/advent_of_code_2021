use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::fmt::{Debug, Display, Formatter};
use std::io;
use std::io::BufRead;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Amphipod {
    A,
    B,
    C,
    D,
}

impl Amphipod {
    fn cost(&self) -> usize {
        match self {
            Amphipod::A => 1,
            Amphipod::B => 10,
            Amphipod::C => 100,
            Amphipod::D => 1000,
        }
    }
}

impl TryFrom<char> for Amphipod {
    type Error = ();

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            'A' => Ok(Self::A),
            'B' => Ok(Self::B),
            'C' => Ok(Self::C),
            'D' => Ok(Self::D),
            _ => Err(()),
        }
    }
}

#[derive(Clone)]
struct Game {
    destinations: HashMap<usize, Amphipod>,
    amphipods: HashMap<(usize, usize), Amphipod>,
    energy: usize,
}

impl PartialEq<Self> for Game {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl PartialOrd<Self> for Game {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Game {}

impl Ord for Game {
    fn cmp(&self, other: &Self) -> Ordering {
        self.destinations
            .len()
            .cmp(&other.destinations.len())
            .then(self.energy.cmp(&other.energy).reverse())
            .then(self.done().cmp(&other.done()))
            .then(format!("{:?}", self).cmp(&format!("{:?}", other)))
    }
}

impl Debug for Game {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut d = self.destinations.iter().collect::<Vec<_>>();
        d.sort_by_key(|(&x, &a)| (a, x));
        let mut a = self.amphipods.iter().collect::<Vec<_>>();
        a.sort_by_key(|(&(x, y), &a)| (a, x, y));

        f.write_str(&format!("{:?} {:?} {:?} {}", d, a, self.energy, self.winner()))
    }
}

impl Display for Game {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut template = (r#"#############
#...........#
###.#.#.#.###
  #.#.#.#.#__
  #.#.#.#.#__
  #.#.#.#.#__
  #########"#)
            .chars()
            .collect::<Vec<_>>();
        for ((x, y), amphipod) in &self.amphipods {
            template[y * 14 + x] = format!("{:?}", amphipod).chars().next().unwrap();
        }
        std::fmt::Display::fmt(&String::from_iter(template), f)
    }
}

impl Game {
    fn state(&self) -> String {
        let mut d = self.destinations.iter().collect::<Vec<_>>();
        d.sort_by_key(|(&x, &a)| (a, x));
        let mut a = self.amphipods.iter().collect::<Vec<_>>();
        a.sort_by_key(|(&(x, y), &a)| (a, x, y));
        format!("{:?} {:?}", d, a)
    }

    fn done(&self) -> i32 {
        self.amphipods
            .iter()
            .map(|(&(x, _), amphipod)| {
                if let Some(dest) = self.destinations.get(&x) {
                    if dest == amphipod {
                        2
                    } else {
                        -1
                    }
                } else {
                    0
                }
            })
            .sum::<i32>()
    }
    fn winner(&self) -> bool {
        self.destinations.len() == 4 && self.done() == 32
    }
    fn next(self) -> impl Iterator<Item = Game> + 'static {
        self.amphipods
            .clone()
            .into_iter()
            .filter_map(move |((x, y), amphipod)| {
                if y == 5 && self.destinations.get(&x) == Some(&amphipod) {
                    None
                } else {
                    let mut next_games = vec![];
                    let mut new_game = self.clone();
                    new_game.amphipods.remove(&(x, y));

                    new_game.to_hallway_moves(x, y, amphipod, |dx, dy| {
                        let mut next_game = new_game.clone();
                        let moves = dx.max(x) - dx.min(x) + dy.max(y) - dy.min(y);
                        next_game.amphipods.insert((dx, dy), amphipod);
                        next_game.energy += moves * amphipod.cost();
                        next_games.push(next_game)
                    });
                    new_game.to_room_moves(x, y, amphipod, |dx, dy| {
                        let mut next_game = new_game.clone();
                        let moves = dx.max(x) - dx.min(x) + dy.max(y) - dy.min(y);
                        next_game.amphipods.insert((dx, dy), amphipod);
                        next_game.energy += moves * amphipod.cost();
                        next_game.destinations.insert(dx, amphipod);
                        next_games.push(next_game);
                    });

                    Some(next_games)
                }
            })
            .flatten()
    }

    fn to_hallway_moves(
        &self,
        x: usize,
        y: usize,
        amphipod: Amphipod,
        mut f: impl FnMut(usize, usize),
    ) {
        if y == 1
            || (y > 2 && (2..y).any(|y| self.amphipods.contains_key(&(x, y))))
            || (self.destinations.get(&x) == Some(&amphipod)
                && (y + 1..=5).all(|y| self.amphipods.get(&(x, y)) == Some(&amphipod)))
        {
            return;
        }

        for direction in [(1..=x - 1).rev().collect::<Vec<usize>>(), (x + 1..=11).collect()] {
            for dest_x in direction {
                if [3, 5, 7, 9].contains(&dest_x) {
                    continue;
                }
                if self.amphipods.contains_key(&(dest_x, 1)) {
                    break;
                }
                f(dest_x, 1)
            }
        }
    }

    fn to_room_moves(
        &self,
        x: usize,
        y: usize,
        amphipod: Amphipod,
        mut f: impl FnMut(usize, usize),
    ) {
        if y >= 2 {
            return;
        }

        let reserved_room = self
            .destinations
            .iter()
            .find_map(|(&dest_x, &value)| if value == amphipod { Some(dest_x) } else { None })
            .unwrap();

        for direction in [(x + 1..=9).collect::<Vec<usize>>(), (3..=x - 1).rev().collect()] {
            for dest_x in direction {
                if self.amphipods.contains_key(&(dest_x, 1)) {
                    break;
                }

                if reserved_room == dest_x
                    && ((3..=5).all(|y| {
                        !self.amphipods.contains_key(&(dest_x, y))
                            || self.amphipods.get(&(dest_x, y)) == Some(&amphipod)
                    }))
                {
                    let dest_y = (2..=5)
                        .rev()
                        .find(|y| !self.amphipods.contains_key(&(dest_x, *y)))
                        .unwrap();
                    f(dest_x, dest_y)
                }
            }
        }
    }
}

fn main() {
    let stdin = io::stdin();
    let mut amphipods = stdin.lock().lines().flatten().enumerate().fold(
        HashMap::new(),
        |mut amphipods, (y, line)| {
            for (x, c) in line.chars().enumerate() {
                if let Ok(amphipod) = Amphipod::try_from(c) {
                    amphipods.insert((x, y), amphipod);
                };
            }

            amphipods
        },
    );
    for x in [3, 5, 7, 9] {
        amphipods.insert((x, 5), *amphipods.get(&(x, 3)).unwrap());
    }

    for (x, y, c) in [
        (3, 3, 'D'),
        (3, 4, 'D'),
        (5, 3, 'C'),
        (5, 4, 'B'),
        (7, 3, 'B'),
        (7, 4, 'A'),
        (9, 3, 'A'),
        (9, 4, 'C'),
    ] {
        amphipods.insert((x, y), Amphipod::try_from(c).unwrap());
    }

    let game = Game {
        destinations: HashMap::from([
            (3, Amphipod::A),
            (5, Amphipod::B),
            (7, Amphipod::C),
            (9, Amphipod::D),
        ]),
        amphipods,
        energy: 0,
    };

    let mut best = usize::MAX;

    let mut next_games = BinaryHeap::<Game>::new();
    next_games.push(game);

    let mut seen = HashMap::<String, usize>::new();
    while let Some(game) = next_games.pop() {
        if game.winner() {
            if best > game.energy {
                best = game.energy;
            }
            continue;
        } else if best > game.energy {
            next_games.extend(game.clone().next().filter(|game| {
                if game.energy >= best {
                    false
                } else {
                    let state = game.state();
                    if let Some(energy) = seen.get_mut(&state) {
                        if *energy > game.energy {
                            *energy = game.energy;
                            true
                        } else {
                            false
                        }
                    } else {
                        seen.insert(state, game.energy);
                        true
                    }
                }
            }))
        }
    }
    println!("{}", best);
}
