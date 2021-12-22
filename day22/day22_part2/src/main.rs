use std::cmp::Ordering;
use std::collections::HashSet;
use std::io;
use std::io::BufRead;
use std::ops::RangeInclusive;
use std::str::FromStr;

#[derive(Debug)]
enum Direction {
    X,
    Y,
    Z,
}

impl FromStr for Direction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.chars().next() {
            Some('x') => Ok(Direction::X),
            Some('y') => Ok(Direction::Y),
            Some('z') => Ok(Direction::Z),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
struct Step {
    direction: Direction,
    range: RangeInclusive<i64>,
}

impl FromStr for Step {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let direction = s.parse().or(Err("invalid direction"))?;
        let from = s[2..]
            .split('.')
            .next()
            .ok_or("cannot parse 'from'")?
            .parse::<i64>()
            .or(Err("invalid integer"))?;
        let to = s[2..]
            .rsplit('.')
            .next()
            .ok_or("cannot parse 'to'")?
            .parse::<i64>()
            .or(Err("invalid integer"))?;

        Ok(Self { direction, range: (from..=to) })
    }
}

#[derive(Debug)]
struct Instruction {
    on: bool,
    volume: Volume,
}

impl FromStr for Instruction {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let on = if s[0..=2].eq("on ") {
            true
        } else if s[0..=3].eq("off ") {
            false
        } else {
            return Err("invalid switch");
        };

        let steps = s
            .split_once(' ')
            .ok_or("cannot split line")?
            .1
            .split(',')
            .map(str::parse)
            .collect::<Result<Vec<Step>, _>>()?;

        let mut range_x = None;
        let mut range_y = None;
        let mut range_z = None;

        for step in steps {
            match step.direction {
                Direction::X => range_x = Some(step.range),
                Direction::Y => range_y = Some(step.range),
                Direction::Z => range_z = Some(step.range),
            }
        }

        Ok(Self {
            on,
            volume: Volume {
                x: range_x.ok_or("no x range")?,
                y: range_y.ok_or("no y range")?,
                z: range_z.ok_or("no z range")?,
            },
        })
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
struct Volume {
    x: RangeInclusive<i64>,
    y: RangeInclusive<i64>,
    z: RangeInclusive<i64>,
}

impl PartialOrd for Volume {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        (self.x.start(), self.y.start(), self.z.start()).partial_cmp(&(
            other.x.start(),
            other.y.start(),
            other.z.start(),
        ))
    }
}

impl Ord for Volume {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl Volume {
    fn intersects(&self, volume: &Volume) -> bool {
        volume.x.start() <= self.x.end()
            && self.x.start() <= volume.x.end()
            && volume.y.start() <= self.y.end()
            && self.y.start() <= volume.y.end()
            && volume.z.start() <= self.z.end()
            && self.z.start() <= volume.z.end()
    }

    fn cut_x(self, x: i64) -> Vec<Volume> {
        if self.x.contains(&x) {
            vec![
                Volume { x: *self.x.start()..=x - 1, y: self.y.clone(), z: self.z.clone() },
                Volume { x: x..=*self.x.end(), y: self.y, z: self.z },
            ]
        } else {
            vec![self]
        }
    }

    fn cut_y(self, y: i64) -> Vec<Volume> {
        if self.y.contains(&y) {
            vec![
                Volume { x: self.x.clone(), y: *self.y.start()..=y - 1, z: self.z.clone() },
                Volume { x: self.x, y: y..=*self.y.end(), z: self.z },
            ]
        } else {
            vec![self]
        }
    }

    fn cut_z(self, z: i64) -> Vec<Volume> {
        if self.z.contains(&z) {
            vec![
                Volume { x: self.x.clone(), y: self.y.clone(), z: *self.z.start()..=z - 1 },
                Volume { x: self.x, y: self.y, z: z..=*self.z.end() },
            ]
        } else {
            vec![self]
        }
    }

    fn cut(self, volume: &Volume) -> Vec<Volume> {
        self.cut_x(*volume.x.start())
            .into_iter()
            .map(|part| part.cut_x(*volume.x.end() + 1))
            .flatten()
            .map(|part| part.cut_y(*volume.y.start()))
            .flatten()
            .map(|part| part.cut_y(*volume.y.end() + 1))
            .flatten()
            .map(|part| part.cut_z(*volume.z.start()))
            .flatten()
            .map(|part| part.cut_z(*volume.z.end() + 1))
            .flatten()
            .filter(|volume| volume.volume() > 0)
            .collect()
    }

    fn volume(&self) -> usize {
        return ((*self.x.end() - *self.x.start() + 1)
            * (*self.y.end() - *self.y.start() + 1)
            * (*self.z.end() - *self.z.start() + 1)) as usize;
    }
}

fn main() -> Result<(), &'static str> {
    let stdin = io::stdin();
    let instructions = stdin
        .lock()
        .lines()
        .flatten()
        .map(|line| str::parse(&line))
        .collect::<Result<Vec<Instruction>, _>>()?;

    let mut grid = HashSet::new();
    for instruction in instructions {
        let intersections = grid
            .iter()
            .filter(|volume| instruction.volume.intersects(volume))
            .cloned()
            .collect::<Vec<Volume>>();
        let mut instruction_parts = vec![instruction.volume.clone()];
        for volume in intersections {
            grid.remove(&volume);

            if instruction.on {
                instruction_parts =
                    instruction_parts.into_iter().map(|part| part.cut(&volume)).flatten().collect();
            }

            let volume_parts = volume.cut(&instruction.volume);
            for volume in volume_parts {
                if !volume.intersects(&instruction.volume) {
                    grid.insert(volume);
                }
            }
        }

        if instruction.on {
            for part in instruction_parts {
                grid.insert(part);
            }
        }
    }

    let cubes_on = grid.into_iter().map(|volume| volume.volume()).sum::<usize>();
    println!("{}", cubes_on);
    Ok(())
}
