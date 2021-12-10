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
        ')' => 1,
        ']' => 2,
        '}' => 3,
        '>' => 4
    };

    let closings: HashMap<char, char> = openings.iter().map(|(&x, &y)| (y, x)).collect();

    let mut scores_by_line: Vec<usize> = vec![];

    'main: for line in lines {
        let mut stack = vec![];

        for c in line.chars() {
            if let Some(&closing) = openings.get(&c) {
                stack.push(closing)
            } else if closings.contains_key(&c) {
                match stack.pop() {
                    None => {
                        continue 'main;
                    }
                    Some(expected_closing) => {
                        if c != expected_closing {
                            continue 'main;
                        }
                    }
                }
            }
        }
        let mut score: usize = 0;
        while let Some(c) = stack.pop() {
            score *= 5;
            score += scores.get(&c).unwrap();
        }
        scores_by_line.push(score)
    }
    scores_by_line.sort_unstable();
    let final_score = scores_by_line.get(scores_by_line.len() / 2).unwrap();
    println!("{}", final_score);
}
