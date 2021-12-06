use std::collections::HashMap;
use std::io;
use std::io::BufRead;

fn main() {
    let stdin = io::stdin();
    let line = stdin.lock().lines().flatten().next().unwrap();
    let ages: Vec<usize> = line.split(',').map(str::parse).flatten().collect();

    let mut counts_per_age: HashMap<usize, usize> = HashMap::new();
    for age in ages {
        *counts_per_age.entry(age).or_default() += 1;
    }

    for _ in 0..256 {
        let mut next = HashMap::new();

        for (age, count) in counts_per_age {
            match age {
                0 => {
                    *next.entry(6).or_default() += count;
                    next.insert(8, count);
                }
                _ => {
                    *next.entry(age - 1).or_default() += count;
                }
            }
        }
        counts_per_age = next;
    }

    println!("{}", counts_per_age.values().sum::<usize>());
}
