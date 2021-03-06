use itertools::Itertools;
use std::io::{stdin, BufRead};

fn visit(
    paths: &[Vec<i32>],
    current: i32,
    visited: &mut [u8],
    visited_small_cave: bool,
    counter: &mut i32,
) {
    for &destination in &paths[current.abs() as usize] {
        if destination == 0 {
            *counter += 1;
            continue;
        }
        let visited_small_cave = if destination.is_positive() {
            visited_small_cave
        } else {
            match (visited_small_cave, visited[-destination as usize]) {
                (_, 0) => visited_small_cave,
                (false, 1) => true,
                _ => continue,
            }
        };
        visited[destination.abs() as usize] += 1;
        visit(paths, destination, visited, visited_small_cave, counter);
        visited[destination.abs() as usize] -= 1;
    }
}

fn make_id(s: &str) -> i32 {
    if s.eq("end") {
        return 0;
    }
    if s.eq("start") {
        return make_id("st");
    }
    let small = s.ge("a");
    let result = s
        .bytes()
        .map(|c| c.to_ascii_uppercase())
        .fold(0, |acc: i32, x| acc * 16 + (x - b'A') as i32);
    if small {
        -result
    } else {
        result
    }
}

fn main() {
    let start: i32 = make_id("start");
    let stdin = stdin();
    let paths = stdin
        .lock()
        .lines()
        .flatten()
        .map(|x| {
            x.split_once('-').map(|(source, destination)| {
                let source = make_id(source);
                let destination = make_id(destination);

                vec![(source, destination), (destination, source)]
            })
        })
        .flatten()
        .flatten()
        .filter(|&(_, destination)| destination != start)
        .sorted()
        .into_group_map();

    let mut counter = 0;
    let mut visited = [0u8; 1000];
    let mut paths_reserve = Vec::new();
    for _ in 0..=1000 {
        paths_reserve.push(Vec::new());
    }

    let fixed_paths = paths_reserve.as_mut_slice();

    for (source, destinations) in paths {
        fixed_paths[source.abs() as usize] = destinations
    }

    visit(fixed_paths, start, &mut visited, false, &mut counter);

    println!("{}", counter)
}
