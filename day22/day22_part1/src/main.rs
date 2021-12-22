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
    range: RangeInclusive<i32>,
}

impl FromStr for Step {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let direction = s.parse().or(Err("invalid direction"))?;
        let from = s[2..]
            .split('.')
            .next()
            .ok_or("cannot parse 'from'")?
            .parse::<i32>()
            .or(Err("invalid integer"))?;
        let to = s[2..]
            .rsplit('.')
            .next()
            .ok_or("cannot parse 'to'")?
            .parse::<i32>()
            .or(Err("invalid integer"))?;

        Ok(Self { direction, range: (from..=to) })
    }
}

#[derive(Debug)]
struct Instruction {
    on: bool,
    x: RangeInclusive<i32>,
    y: RangeInclusive<i32>,
    z: RangeInclusive<i32>,
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
            x: range_x.ok_or("no x range")?,
            y: range_y.ok_or("no y range")?,
            z: range_z.ok_or("no z range")?,
        })
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
        if *instruction.x.start() >= -50
            && *instruction.x.end() <= 50
            && *instruction.y.start() >= -50
            && *instruction.y.end() <= 50
            && *instruction.z.start() >= -50
            && *instruction.z.end() <= 50
        {
            for x in instruction.x {
                for y in instruction.y.clone() {
                    for z in instruction.z.clone() {
                        if instruction.on {
                            grid.insert((x, y, z));
                        } else {
                            grid.remove(&(x, y, z));
                        }
                    }
                }
            }
        }
    }
    println!("{}", grid.len());
    Ok(())
}
