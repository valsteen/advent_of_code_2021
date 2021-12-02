use std::borrow::Borrow;
use std::io;
use std::io::BufRead;
use std::str::FromStr;

enum Action {
    Forward(i32),
    Up(i32),
    Down(i32),
}

impl FromStr for Action {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (direction, value) = s.split_once(" ").ok_or(())?;
        let value: i32 = str::parse(value).map_err(|_| ())?;
        match direction {
            s if s.eq("forward") => Ok(Self::Forward(value)),
            s if s.eq("up") => Ok(Self::Up(value)),
            s if s.eq("down") => Ok(Self::Down(value)),
            _ => Err(()),
        }
    }
}

fn main() {
    let stdin = io::stdin();
    let actions = stdin
        .lock()
        .lines()
        .flatten()
        .map(|s| Action::from_str(s.borrow()))
        .flatten();

    let mut aim = 0 ;
    let mut depth = 0;
    let mut position = 0;

    for a in actions {
        match a {
            Action::Forward(value) => {
                position += value;
                depth += aim * value;
            }
            Action::Up(value) => {
                aim -= value;
            }
            Action::Down(value) => aim += value,
        }
    }

    println!("{} {} {}", depth, position, depth * position);
}
