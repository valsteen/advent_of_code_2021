use itertools::Itertools;
use maplit::hashmap;
use std::collections::HashMap;
use std::io;
use std::io::Read;
use std::ops::ControlFlow;
use std::ops::ControlFlow::{Break, Continue};

fn permutations<F: FnMut(Vec<u8>) -> ControlFlow<Vec<usize>>>(
    result: Vec<u8>,
    wire_to_segment: Vec<Vec<u8>>,
    f: &mut F,
) -> ControlFlow<Vec<usize>> {
    let (segments, wire_to_segment) = match wire_to_segment.split_first() {
        None => {
            f(result)?;
            return Continue(());
        }
        Some((segments, wire_to_segment)) => (segments, wire_to_segment),
    };

    for &segment in segments {
        let mut result = result.clone();
        let mut wire_to_segment = wire_to_segment.to_vec();

        for segments in wire_to_segment.iter_mut() {
            segments.retain(|&segment_| segment_ != segment);
        }

        result.push(segment);
        permutations(result, wire_to_segment, f)?;
    }

    Continue(())
}

fn main() {
    let digit_to_segments: Vec<Vec<u8>> = vec![
        vec![0, 1, 2, 4, 5, 6],
        vec![2, 5],
        vec![0, 2, 3, 4, 6],
        vec![0, 2, 3, 5, 6],
        vec![1, 2, 3, 5],
        vec![0, 1, 3, 5, 6],
        vec![0, 1, 3, 4, 5, 6],
        vec![0, 2, 5],
        vec![0, 1, 2, 3, 4, 5, 6],
        vec![0, 1, 2, 3, 5, 6],
    ];
    let length_to_digit: HashMap<usize, u8> = hashmap! {
        2 => 1,
        4 => 4,
        3 => 7,
        7 => 8,
    };

    let stdin = io::stdin();
    let lines: String = stdin.lock().bytes().flatten().map(char::from).collect();

    #[allow(clippy::type_complexity)]
    let lines: Vec<(Vec<Vec<u8>>, Vec<Vec<u8>>)> = lines
        .replace("|\n", "")
        .replace(" | ", " ")
        .split('\n')
        .map(|line| {
            line.split(' ')
                .map(|pattern| pattern.bytes().map(|digit| digit - b'a').sorted().collect())
                .collect::<Vec<Vec<u8>>>()
        })
        .filter(|line| line.len() > 1)
        .map(|line| {
            let (patterns, display) = line.split_at(10);
            (patterns.to_vec(), display.to_vec())
        })
        .collect();

    let mut result = vec![];

    for (patterns, displayed_patterns) in &lines {
        let mut wire_to_segments: Vec<Vec<u8>> = (0..=6).map(|_| ((0..=6).collect())).collect();

        for pattern in patterns.iter() {
            if let Some(digit) = length_to_digit.get(&pattern.len()) {
                let segments_for_digit = digit_to_segments.get(*digit as usize).unwrap();
                for (wire, possible_segments) in wire_to_segments.iter_mut().enumerate() {
                    if pattern.contains(&(wire as u8)) {
                        // wires appearing in the current pattern can only belong to one of the segments for the current digit
                        possible_segments.retain(|segment| segments_for_digit.contains(segment));
                    } else {
                        // wires not appearing in the current pattern cannot belong to the segments of the digit
                        possible_segments.retain(|segment| !segments_for_digit.contains(segment));
                    }
                }
            }
        }

        let pattern_indexes_to_digit = if let Break(value) =
            permutations(vec![], wire_to_segments, &mut |wire_to_segment: Vec<u8>| {
                let pattern_index_to_digit = patterns
                    .iter()
                    .map(|wires| {
                        let segments: Vec<u8> = wires
                            .iter()
                            .map(|wire| *wire_to_segment.get(*wire as usize).unwrap())
                            .sorted()
                            .collect();
                        digit_to_segments.iter().position(|x| x.eq(&segments))
                    })
                    .flatten()
                    .collect_vec();
                if pattern_index_to_digit.len() == patterns.len() {
                    Break(pattern_index_to_digit)
                } else {
                    Continue(())
                }
            }) {
            value
        } else {
            unreachable!()
        };

        let mut number = 0;
        for displayed_pattern in displayed_patterns {
            let pattern_index =
                patterns.iter().position(|pattern| pattern.eq(displayed_pattern)).unwrap();
            let &digit = pattern_indexes_to_digit.get(pattern_index).unwrap();
            number = number * 10 + digit as usize;
        }

        result.push(number);
    }

    println!("{:?}", result.into_iter().sum::<usize>());
}
