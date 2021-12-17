use itertools::Itertools;
use std::cmp::Ordering;
use std::io;
use std::io::Read;

struct Target {
    x_min: i32,
    x_max: i32,
    y_min: i32,
    y_max: i32,
}

#[derive(Debug)]
struct Position {
    x: i32,
    y: i32,
    x_velocity: i32,
    y_velocity: i32,
}

enum Reachability {
    Unreachable,
    Reachable,
    Hit,
}

impl Position {
    fn advance(&mut self) {
        self.x += self.x_velocity;
        self.y += self.y_velocity;

        self.x_velocity -= self.x_velocity.signum();
        self.y_velocity -= 1;
    }

    fn reachability(&self, target: &Target) -> Reachability {
        if (target.x_min..=target.x_max).contains(&self.x)
            && (target.y_min..=target.y_max).contains(&self.y)
        {
            Reachability::Hit
        } else if match self.x_velocity.cmp(&0) {
            Ordering::Equal => {
                if !(target.x_min..=target.x_max).contains(&self.x) {
                    false
                } else if self.y >= target.y_min {
                    true
                } else {
                    self.y_velocity > 0
                }
            }
            Ordering::Less => self.x >= target.x_min,
            Ordering::Greater => self.x <= target.x_max,
        } {
            Reachability::Reachable
        } else {
            Reachability::Unreachable
        }
    }
}

fn main() {
    let stdin = io::stdin();
    let numbers =
        &stdin.lock().bytes().flatten().group_by(|&c| (b'0'..=b'9').contains(&c) || c == b'-');
    let (x_min, x_max, y_min, y_max) = numbers
        .into_iter()
        .map(|(_, chunk)| String::from_iter(chunk.map(char::from)).parse())
        .flatten()
        .tuples()
        .next()
        .unwrap();

    let target = Target { x_min, x_max, y_min, y_max };

    let mut best_y = i32::MIN;

    for x_velocity in 0..x_max {
        let mut y_velocity = 0;

        loop {
            let mut position = Position { x: 0, y: 0, x_velocity, y_velocity };

            let mut max_y = position.y;

            loop {
                match position.reachability(&target) {
                    Reachability::Unreachable => break,
                    Reachability::Reachable => {}
                    Reachability::Hit => {
                        best_y = best_y.max(max_y);
                        break;
                    }
                }
                position.advance();
                max_y = max_y.max(position.y);
            }

            if position.x < target.x_min {
                break;
            }

            if y_velocity > 1000 {
                // lame breaking condition but hey it works
                break;
            }

            y_velocity += 1;
        }
    }

    println!("{:?}", best_y);
}
