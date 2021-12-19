use itertools::Itertools;
use std::borrow::Borrow;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::{Debug, Display, Formatter};
use std::io;
use std::io::BufRead;

#[derive(Clone, PartialEq)]
struct Coordinates {
    x: i32,
    y: i32,
    z: i32,
}

#[derive(Debug)]
struct Point {
    orientations: [Coordinates; 24],
}

impl Point {
    fn new(x: i32, y: i32, z: i32) -> Self {
        Self { orientations: Coordinates { x, y, z }.orientations() }
    }
}

impl Coordinates {
    fn orientations(&self) -> [Coordinates; 24] {
        [
            [-self.x, -self.y, self.z],
            [-self.x, -self.z, -self.y],
            [-self.x, self.y, -self.z],
            [-self.x, self.z, self.y],
            [-self.y, -self.x, -self.z],
            [-self.y, -self.z, self.x],
            [-self.y, self.x, self.z],
            [-self.y, self.z, -self.x],
            [-self.z, -self.x, self.y],
            [-self.z, -self.y, -self.x],
            [-self.z, self.x, -self.y],
            [-self.z, self.y, self.x],
            [self.x, -self.y, -self.z],
            [self.x, -self.z, self.y],
            [self.x, self.y, self.z],
            [self.x, self.z, -self.y],
            [self.y, -self.x, self.z],
            [self.y, -self.z, -self.x],
            [self.y, self.x, -self.z],
            [self.y, self.z, self.x],
            [self.z, -self.x, -self.y],
            [self.z, -self.y, self.x],
            [self.z, self.x, self.y],
            [self.z, self.y, -self.x],
        ]
        .map(|pos| Coordinates::from(&pos[..]))
    }
}

impl Display for Coordinates {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("({},{},{})", self.x, self.y, self.z))
    }
}

impl Debug for Coordinates {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Display for Point {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.orientations, f)
    }
}

impl From<&[i32]> for Coordinates {
    fn from(numbers: &[i32]) -> Self {
        Coordinates { x: numbers[0], y: numbers[1], z: numbers[2] }
    }
}

fn main() {
    let stdin = io::stdin();

    let scanners = stdin.lock().lines().flatten().fold(vec![], |mut acc, line| {
        let groups = line.chars().group_by(|&c| c.is_ascii_digit() || c == '-');
        let numbers = groups
            .borrow()
            .into_iter()
            .map(|(_, digits)| String::from_iter(digits).parse())
            .flatten()
            .collect_vec();
        if numbers.len() == 1 {
            acc.push(vec![]);
        } else if numbers.len() == 3 {
            acc.last_mut().unwrap().push(Point::new(numbers[0], numbers[1], numbers[2]))
        }
        acc
    });

    let mut known_scanner_to_test = VecDeque::new();
    known_scanner_to_test.push_back(0);
    let mut unknown_scanners = (1..scanners.len()).collect_vec();
    let mut orientations = vec![None; scanners.len()];
    let mut distances = vec![None; scanners.len()];
    orientations[0] = Some(0);
    distances[0] = Some((0, 0, 0));
    let mut discovered = HashSet::<_>::from_iter(scanners[0].iter().map(|p| {
        let coordinates = &p.orientations[0];
        (coordinates.x, coordinates.y, coordinates.z)
    }));

    while let Some(known_scanner) = known_scanner_to_test.pop_front() {
        for unknown_scanner in unknown_scanners.clone() {
            let mut scores = vec![];
            for orientation in 0..24 {
                let mut distances = HashMap::<_, usize>::new();
                for known_point in &scanners[known_scanner] {
                    let known_coordinates =
                        &known_point.orientations[orientations[known_scanner].unwrap()];
                    for unknown_point in &scanners[unknown_scanner] {
                        let unknown_coordinates = &unknown_point.orientations[orientation];
                        *distances
                            .entry((
                                known_coordinates.x - unknown_coordinates.x,
                                known_coordinates.y - unknown_coordinates.y,
                                known_coordinates.z - unknown_coordinates.z,
                            ))
                            .or_default() += 1;
                    }
                }
                let (distance, _) = distances.iter().max_by_key(|&(_, &matches)| matches).unwrap();
                scores.push((distances.len(), orientation, *distance))
            }

            scores.sort_unstable_by_key(|(score, ..)| *score);

            for (_, orientation, distance) in scores {
                let mut matches = 0;
                let known_coordinates = scanners[known_scanner]
                    .iter()
                    .map(|p| p.orientations[orientations[known_scanner].unwrap()].clone())
                    .collect_vec();
                for point in &scanners[unknown_scanner] {
                    let mut unknown_coordinates = point.orientations[orientation].clone();
                    unknown_coordinates.x += distance.0;
                    unknown_coordinates.y += distance.1;
                    unknown_coordinates.z += distance.2;
                    if known_coordinates.contains(&unknown_coordinates) {
                        matches += 1;
                    }
                }
                if matches >= 12 {
                    unknown_scanners.retain(|&k| k != unknown_scanner);
                    orientations[unknown_scanner] = Some(orientation);
                    let known_distance = distances[known_scanner].unwrap();
                    distances[unknown_scanner] = Some((
                        -distance.0 + known_distance.0,
                        -distance.1 + known_distance.1,
                        -distance.2 + known_distance.2,
                    ));
                    known_scanner_to_test.push_back(unknown_scanner);

                    for point in scanners[unknown_scanner].iter() {
                        let mut coordinates = point.orientations[orientation].clone();
                        coordinates.x -= -distance.0 + known_distance.0;
                        coordinates.y -= -distance.1 + known_distance.1;
                        coordinates.z -= -distance.2 + known_distance.2;
                        discovered.insert((coordinates.x, coordinates.y, coordinates.z));
                    }
                }
            }
        }
    }

    println!("{:?}", discovered.len());
}
