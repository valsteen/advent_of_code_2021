use itertools::Itertools;
use std::collections::HashMap;
use std::io::{stdin, BufRead};

fn visit(
    paths: &HashMap<i32, Vec<i32>>,
    visited: &mut Vec<i32>,
    visited_small_cave: bool,
    counter: &mut i32,
) {
    let &current = visited.last().unwrap();
    if current == 0 {
        *counter += 1;
        return;
    }
    if let Some(destinations) = paths.get(&current) {
        for (visited_small_cave, destination) in destinations
            .iter()
            .filter_map(|&destination| {
                if destination.is_positive() {
                    Some((visited_small_cave, destination))
                } else {
                    let visit_count =
                        visited.iter().filter(|&cave| cave.eq(&destination)).take(2).count();
                    match (visited_small_cave, visit_count) {
                        (_, 0) => Some((visited_small_cave, destination)),
                        (false, 1) => Some((true, destination)),
                        _ => None,
                    }
                }
            })
            .collect_vec()
        {
            visited.push(destination);
            visit(paths, visited, visited_small_cave, counter);
            visited.pop().unwrap();
        }
    }
}

fn make_id(s: &str) -> i32 {
    if s.eq("end") {
        return 0;
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
    visit(&paths, &mut vec![start], false, &mut counter);

    println!("{}", counter)
}
