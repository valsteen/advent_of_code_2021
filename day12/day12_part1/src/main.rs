use itertools::Itertools;
use std::collections::HashMap;
use std::io::{stdin, BufRead};

fn visit(paths: &HashMap<String, Vec<String>>, visited: &[String], counter: &mut usize) {
    let current = visited.last().unwrap();
    if current.eq("end") {
        *counter += 1;
        return;
    }
    if let Some(destinations) = paths.get(current) {
        for destination in destinations
            .iter()
            .filter(|destination| !(destination.as_str().ge("a") && visited.contains(destination)))
        {
            let mut visited = visited.to_vec();
            visited.push(destination.clone());
            visit(paths, &visited, counter)
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
        .sorted()
        .into_group_map();

    let mut counter = 0;
    visit(&paths, &["start".to_string()], &mut counter);

    println!("{}", counter)
}
