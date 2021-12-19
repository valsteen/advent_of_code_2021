use itertools::Itertools;
use rayon::prelude::*;
use std::borrow::Borrow;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::{Debug, Display, Formatter};
use std::io;
use std::io::BufRead;
use std::sync::Mutex;

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

struct LockedData {
    discovered: HashSet<(i32,i32,i32)>,
    known_scanner_to_test: VecDeque<usize>,
    unknown_scanners: Vec<usize>,
    orientations: Vec<Option<usize>>,
    distances: Vec<Option<(i32,i32,i32)>>
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
    let unknown_scanners = (1..scanners.len()).collect_vec();
    let mut orientations = vec![None; scanners.len()];
    let mut distances =vec![None; scanners.len()];
    orientations[0] = Some(0);
    distances[0] = Some((0, 0, 0));
    let discovered = HashSet::<_>::from_iter(scanners[0].iter().map(|p| {
        let coordinates = &p.orientations[0];
        (coordinates.x, coordinates.y, coordinates.z)
    }));

    let locked_data = Mutex::new(LockedData{
        discovered,
        known_scanner_to_test,
        unknown_scanners,
        orientations,
        distances
    });

    // clippy didn't spot that a while would keep the lock and lead to a deadlock
    #[allow(clippy::while_let_loop)]
    loop {
        let known_scanner = if let Some(known_scanner) = locked_data.lock().unwrap().known_scanner_to_test.pop_front() {
            known_scanner
        } else {
            break
        };
        let unknown_scanners = { locked_data.lock().unwrap().unknown_scanners.clone() };
        unknown_scanners.into_par_iter().for_each(|unknown_scanner| {
            let (orientation, distance) =
                match (0..24).into_par_iter().find_map_first(|orientation| {
                    let distances = scanners[known_scanner]
                        .par_iter()
                        .map(|known_point| {
                            let mut distances = HashMap::<_, usize>::new();
                            let known_coordinates =
                                &known_point.orientations[locked_data.lock().unwrap().orientations[known_scanner].unwrap()];
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
                            distances
                        })
                        .reduce(
                            HashMap::new,
                            |mut acc, distances: HashMap<(i32, i32, i32), usize>| {
                                for (coordinates, matches) in distances.into_iter() {
                                    *acc.entry(coordinates).or_default() += matches
                                }
                                acc
                            },
                        );

                    let (&distance, &k) =
                        distances.iter().max_by_key(|&(_, &matches)| matches).unwrap();
                    if k >= 12 {
                        Some((orientation, distance))
                    } else {
                        None
                    }
                }) {
                    None => return,
                    Some((orientation, distance)) => (orientation, distance),
                };

            {
                let mut locked_data = locked_data.lock().unwrap();
                let known_distance = locked_data.distances[known_scanner].unwrap();
                locked_data.unknown_scanners.retain(|&k| k != unknown_scanner);
                locked_data.orientations[unknown_scanner] = Some(orientation);
                locked_data.distances[known_scanner].unwrap();
                locked_data.distances[unknown_scanner] = Some((
                    -distance.0 + known_distance.0,
                    -distance.1 + known_distance.1,
                    -distance.2 + known_distance.2,
                ));
                locked_data.known_scanner_to_test.push_back(unknown_scanner);


                for point in scanners[unknown_scanner].iter() {
                    let mut coordinates = point.orientations[orientation].clone();
                    coordinates.x -= -distance.0 + known_distance.0;
                    coordinates.y -= -distance.1 + known_distance.1;
                    coordinates.z -= -distance.2 + known_distance.2;
                    locked_data.discovered.insert((coordinates.x, coordinates.y, coordinates.z));
                }
            }
        });
    }

    println!("{:?}", locked_data.lock().unwrap().discovered.len());
}
