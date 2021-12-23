use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::io;
use std::io::BufRead;
use std::ops::RangeInclusive;
use std::str::FromStr;
use rstar::{RTree, AABB, RTreeParams, RTreeObject};

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
                volume_type: VolumeType::Plain
            },
        })
    }
}

enum VolumeType {
    Volumes(Vec<Volume>),
    Plain
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
struct Volume {
    x: RangeInclusive<i64>,
    y: RangeInclusive<i64>,
    z: RangeInclusive<i64>,
    volume_type: VolumeType
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
    fn contains(&self, volume: &Volume) -> bool {
        self.x.start() <= volume.x.start() && self.x.end() >= volume.x.end()
        && self.y.start() <= volume.y.start() && self.y.end() >= volume.y.end()
        && self.z.start() <= volume.z.start() && self.z.end() >= volume.z.end()
    }

    fn matches(&mut self, volume: &Volume) -> Vec<&mut Volume>{
        if self.intersects(volume) {
            match &mut self.volume_type {
                VolumeType::Volumes(volumes) => {
                    volumes.iter_mut().filter_map(|part| {
                        if part.intersects(volume) {
                            Some(part.matches(volume))
                        } else {
                            None
                        }
                    }).flatten().collect()
                }
                VolumeType::Plain => {
                    vec![&mut self]
                }
            }
        } else {
            vec![]
        }
    }

    fn is_empty(&self) -> bool {
        match &self.volume_type {
            VolumeType::Volumes(v) => {
                v.is_empty()
            }
            VolumeType::Plain => {
                false
            }
        }
    }
    fn remove(&mut self, volume: &Volume) {
        self.volume_type = match &self.volume_type {
            VolumeType::Volumes(volumes) => {
                let parts = volumes.into_iter().filter(|mut part| {
                    let is_plain = match &part.volume_type {
                        VolumeType::Volumes(_) => {
                            true
                        }
                        VolumeType::Plain => {
                            false
                        }
                    };
                    if is_plain {
                        !part.intersects(volume)
                    } else {
                        part.remove(volume);
                        part.is_empty()
                    }
                }).collect();
                VolumeType::Volumes(parts)
            }
            VolumeType::Plain => unreachable!()
        }
    }

    fn intersects(&self, volume: &Volume) -> bool {
        volume.x.start() <= self.x.end()
            && self.x.start() <= volume.x.end()
            && volume.y.start() <= self.y.end()
            && self.y.start() <= volume.y.end()
            && volume.z.start() <= self.z.end()
            && self.z.start() <= volume.z.end()
    }

    fn cut_x(&mut self, x: i64) {
        if self.x.contains(&x) {
            self.volume_type = match &self.volume_type {
                VolumeType::Volumes(mut volumes) => {
                    for volume in &mut volumes {
                        volume.cut_x(x);
                    }
                    VolumeType::Volumes(volumes)
                }
                VolumeType::Plain => {
                    VolumeType::Volumes(
                        vec![
                            Volume { x: *self.x.start()..=x - 1, y: self.y.clone(), z: self.z.clone(), volume_type: VolumeType::Plain },
                            Volume { x: x..=*self.x.end(), y: self.y.clone(), z: self.z.clone(), volume_type: VolumeType::Plain }
                        ],
                    )
                }
            }
        }
    }

    fn cut_y(&mut self, y: i64) {
        if self.y.contains(&y) {
            self.volume_type = match &self.volume_type {
                VolumeType::Volumes(mut volumes) => {
                    for volume in &mut volumes {
                        volume.cut_y(y);
                    }
                    VolumeType::Volumes(volumes)
                }
                VolumeType::Plain => {
                    VolumeType::Volumes(
                        vec![
                            Volume { x: self.x.clone(), y: *self.y.start()..=y - 1, z: self.z.clone(), volume_type: VolumeType::Plain },
                            Volume { x: self.x.clone(), y: y..=*self.y.end(), z: self.z.clone(), volume_type: VolumeType::Plain },
                        ],
                    )
                }
            }
        }
    }

    fn cut_z(&mut self, z: i64) {
        if self.z.contains(&z) {
            self.volume_type = match &self.volume_type {
                VolumeType::Volumes(mut volumes) => {
                    for volume in &mut volumes {
                        volume.cut_z(z);
                    }
                    VolumeType::Volumes(volumes)
                }
                VolumeType::Plain => {
                    VolumeType::Volumes(
                        vec![
                            Volume { x: self.x.clone(), y: self.y.clone(), z: *self.z.start()..=z - 1, volume_type: VolumeType::Plain },
                            Volume { x: self.x.clone(), y: self.y.clone(), z: z..=*self.z.end(), volume_type: VolumeType::Plain },
                        ],
                    )
                }
            }
        }
    }

    fn cut(&mut self, volume: &Volume) {
        self.cut_x(*volume.x.start());
        self.cut_x(*volume.x.end() + 1);
        self.cut_y(*volume.y.start());
        self.cut_y(*volume.y.end() + 1);
        self.cut_z(*volume.z.start());
        self.cut_z(*volume.z.end() + 1);
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

    let mut grid = Volume {
        x: i64::MIN..=i64::MAX,
        y: i64::MIN..=i64::MAX,
        z: i64::MIN..=i64::MAX,
        volume_type: VolumeType::Volumes(vec![])
    };

    for mut instruction in instructions {
        let intersections = grid.matches(&instruction.volume);

        for volume in intersections {
            if instruction.on {
                instruction.volume.cut(&volume);
            }

            volume.cut(&instruction.volume);
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

    let cubes_on = grid.iter().map(|volume| volume.volume()).sum::<usize>();
    println!("{}", cubes_on);
    Ok(())
}
