use std::collections::HashMap;
use std::io::{stdin, BufRead};

fn main() {
    let stdin = stdin();
    let mut lines = stdin.lock().lines().flatten();

    let line = lines.next().unwrap();
    let mut line = line.chars();

    let mut pairs = HashMap::<String, usize>::new();
    let mut counts = HashMap::<char, usize>::new();

    let mut previous_element = line.next().unwrap();

    *counts.entry(previous_element).or_default() += 1;

    for element in line {
        *counts.entry(element).or_default() += 1;

        let pair = format!("{}{}", previous_element, element);
        *pairs.entry(pair).or_default() += 1;
        previous_element = element;
    }

    let rules: HashMap<_, _> = lines
        .skip(1)
        .map(|s| {
            let mut s = s.chars();
            (format!("{}{}", s.next().unwrap(), s.next().unwrap()), s.nth(4).unwrap())
        })
        .collect();

    for _ in 0..40 {
        let mut next_pairs = HashMap::<String, usize>::new();
        for (pair, occurences) in pairs {
            if let Some(&new_element) = rules.get(&pair) {
                *counts.entry(new_element).or_default() += occurences;

                let mut chars = pair.chars();
                let element_1 = chars.next().unwrap();
                let element_2 = chars.next().unwrap();

                let new_pair_1 = format!("{}{}", element_1, new_element);
                let new_pair_2 = format!("{}{}", new_element, element_2);

                *next_pairs.entry(new_pair_1).or_default() += occurences;
                *next_pairs.entry(new_pair_2).or_default() += occurences;
            } else {
                *next_pairs.entry(pair).or_default() += occurences;
            }
        }
        pairs = next_pairs;
    }
    let mut min = usize::MAX;
    let mut max = 0;

    for &count in counts.values() {
        min = min.min(count);
        max = max.max(count);
    }
    println!("{}-{}={}", max, min, max - min);
}
