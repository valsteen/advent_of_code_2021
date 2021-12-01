use itertools::Itertools;
use std::io;
use std::io::BufRead;

fn main() {
    let stdin = io::stdin();
    let measurements = stdin
        .lock()
        .lines()
        .flatten()
        .map(|x| x.parse::<i32>().unwrap())
        .collect_vec();
    let windows = measurements.windows(3);

    let increases = windows
        .scan(None, |previous: &mut Option<i32>, current| {
            let sum = current.iter().sum();
            let is_increase = match previous {
                None => false,
                Some(value) => sum > *value,
            };
            *previous = Some(sum);
            Some(is_increase)
        })
        .filter(|e| *e)
        .count();

    println!("{}", increases);
}
