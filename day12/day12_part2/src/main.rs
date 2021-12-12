use itertools::Itertools;
use std::collections::HashMap;
use std::io::{stdin, BufRead};

fn visit(
    paths: &HashMap<String, Vec<String>>,
    visited: &[String],
    visited_small_cave: Option<String>,
    counter: &mut usize,
) {
    let current = visited.last().unwrap();
    if current.eq("end") {
        *counter += 1;
        return;
    }
    if let Some(destinations) = paths.get(current) {
        for (visited_small_cave, destination) in destinations.iter().filter_map(|destination| {
            if destination.as_str().lt("a") {
                Some((visited_small_cave.clone(), destination))
            } else {
                let visit_count = visited.iter().filter(|&cave| cave.eq(destination)).take(2).count();
                match (visit_count, &visited_small_cave) {
                    (0, _) => Some((visited_small_cave.clone(), destination)),
                    (1, None) => Some((Some(destination.clone()), destination)),
                    (1, Some(_)) => None,
                    _ => None,
                }
            }
        }) {
            let mut visited = visited.to_vec();
            visited.push(destination.clone());
            visit(paths, &visited, visited_small_cave, counter)
        }
    }
}

fn main() {
    let stdin = stdin();
    let paths = stdin
        .lock()
        .lines()
        .flatten()
        .map(|x| {
            x.split_once('-').map(|(source, destination)| {
                vec![
                    (source.to_string(), destination.to_string()),
                    (destination.to_string(), source.to_string()),
                ]
            })
        })
        .flatten()
        .flatten()
        .filter(|(_, destination)| !destination.eq("start"))
        .sorted()
        .into_group_map();

    let mut counter = 0;
    visit(&paths, &["start".to_string()], None, &mut counter);

    println!("{}", counter)
}
