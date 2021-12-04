use std::io;
use std::io::BufRead;

fn main() {
    let stdin = io::stdin();
    let measurements = stdin.lock().lines().flatten().map(|x| x.parse::<i32>().unwrap());

    let increases = measurements
        .scan(None, |previous, current| {
            let is_increase = match previous {
                None => false,
                Some(value) => current > *value,
            };
            *previous = Some(current);
            Some(is_increase)
        })
        .filter(|is_increase| *is_increase)
        .count();

    println!("{}", increases);
}
