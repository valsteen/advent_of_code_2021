use std::collections::HashMap;
use std::io::{stdin, BufRead};

fn main() {
    let stdin = stdin();
    let mut lines = stdin.lock().lines().flatten();

    let first_line = lines.next().unwrap();

    let mut counts = HashMap::<char, usize>::new();
    let mut pairs = HashMap::<&str, usize>::new();

    let mut rules: HashMap<_, _> = lines
        .skip(1)
        .map(|s| {
            let mut s = s.chars();
            let element_1 = s.next().unwrap();
            let element_2 = s.next().unwrap();
            let new_element = s.nth(4).unwrap();

            let pair = format!("{}{}", element_1, element_2);
            let new_pair_1 = format!("{}{}", element_1, new_element);
            let new_pair_2 = format!("{}{}", new_element, element_2);
            (pair, (new_element, vec![new_pair_1, new_pair_2]))
        })
        .collect();

    let known_pairs = rules.keys().cloned().collect::<Vec<_>>();
    for (_, new_pairs) in rules.values_mut() {
        new_pairs.retain(|pair| known_pairs.contains(pair));
    }

    let mut formula = first_line.chars();
    let mut previous_element = formula.next().unwrap();

    for element in formula {
        *counts.entry(previous_element).or_default() += 1;

        let pair = format!("{}{}", previous_element, element);

        if let Some((key, _)) = rules.get_key_value(&pair) {
            *pairs.entry(key).or_default() += 1;
        }

        previous_element = element;
    }

    *counts.entry(previous_element).or_default() += 1;

    for _ in 0..40 {
        let mut next_pairs = HashMap::new();
        for (pair, occurences) in pairs {
            let (new_element, new_pairs) = rules.get(pair).unwrap();
            *counts.entry(*new_element).or_default() += occurences;

            for new_pair in new_pairs {
                *next_pairs.entry(new_pair.as_str()).or_default() += occurences;
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
