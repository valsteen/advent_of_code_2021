use maplit::hashmap;
use std::collections::HashMap;
use std::io;
use std::io::BufRead;

fn main() {
    let stdin = io::stdin();
    let lines = stdin.lock().lines().flatten();

    let openings = hashmap! {
        '<' => '>',
        '{' => '}',
        '[' => ']',
        '(' => ')'
    };

    let scores = hashmap! {
        ')' => 3,
        ']' => 57,
        '}' => 1197,
        '>' => 25137
    };

    let closings: HashMap<char, char> = openings.iter().map(|(&x, &y)| (y, x)).collect();

    let mut score = 0;

    for line in lines {
        let mut stack = vec![];

        for c in line.chars() {
            if let Some(&closing) = openings.get(&c) {
                stack.push(closing)
            } else if closings.contains_key(&c) {
                match stack.pop() {
                    None => continue,
                    Some(expected_closing) => {
                        if c != expected_closing {
                            score += scores.get(&c).unwrap();
                            continue;
                        }
                    }
                }
            }
        }
    }
    println!("{}", score);
}
