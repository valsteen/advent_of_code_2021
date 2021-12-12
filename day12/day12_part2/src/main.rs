use itertools::Itertools;
use std::collections::HashMap;
use std::io::{stdin, BufRead};

fn visit<'a>(
    paths: &'a HashMap<String, Vec<String>>,
    visited: &mut Vec<&'a str>,
    visited_small_cave: Option<&'a str>,
    counter: &mut usize,
) {
    let &current = visited.last().unwrap();
    if current.eq("end") {
        *counter += 1;
        return;
    }
    if let Some(destinations) = paths.get(current) {
        for (visited_small_cave, destination) in destinations
            .iter()
            .filter_map(|destination| {
                if destination.as_str().lt("a") {
                    Some((visited_small_cave, destination))
                } else {
                    let visit_count =
                        visited.iter().filter(|&cave| cave.eq(destination)).take(2).count();
                    match (visit_count, &visited_small_cave) {
                        (0, _) => Some((visited_small_cave, destination)),
                        (1, None) => Some((Some(destination), destination)),
                        (1, Some(_)) => None,
                        _ => None,
                    }
                }
            })
            .collect_vec()
        {
            visited.push(destination.as_str());
            visit(paths, visited, visited_small_cave, counter);
            visited.pop().unwrap();
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
    visit(&paths, &mut vec!["start"], None, &mut counter);

    println!("{}", counter)
}
