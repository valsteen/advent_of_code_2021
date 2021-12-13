use itertools::Itertools;
use std::collections::HashSet;
use std::io::{stdin, BufRead};

fn fold(map: &mut HashSet<(i32, i32)>, direction: char, at: i32) {
    let folded_dots = map
        .iter()
        .filter(|(x, y)| match direction {
            'x' => *x >= at,
            'y' => *y >= at,
            _ => unreachable!(),
        })
        .copied()
        .collect_vec();

    for (x, y) in folded_dots {
        map.remove(&(x, y));
        let target = match direction {
            'x' => ((at * 2 - x), y),
            'y' => (x, (at * 2 - y)),
            _ => unreachable!(),
        };
        map.insert(target);
    }
}

fn main() {
    let stdin = stdin();
    let mut lines = stdin.lock().lines().flatten().collect_vec().into_iter();
    let mut map: HashSet<(i32, i32)> = lines
        .take_while_ref(|line| !line.is_empty())
        .map(|line| line.split(',').map(|s| s.parse()).flatten().collect_tuple())
        .flatten()
        .collect();

    let folds: Vec<(char, i32)> = lines
        .skip(1)
        .map(|line| {
            let (direction, amount) = line.rsplit_once(' ').unwrap().1.split_once('=').unwrap();
            (direction.chars().next().unwrap(), amount.parse().unwrap())
        })
        .collect();

    for (direction, at) in folds {
        fold(&mut map, direction, at)
    }

    let &max_x = map.iter().map(|(x, _)| x).max().unwrap();
    let &max_y = map.iter().map(|(_, y)| y).max().unwrap();

    for y in 0..=max_y {
        let line =
            String::from_iter((0..=max_x).map(|x| if map.contains(&(x, y)) { '#' } else { '.' }));
        println!("{}", line);
    }

    println!("{}", map.len());
}
