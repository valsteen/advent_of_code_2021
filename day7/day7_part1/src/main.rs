use itertools::{Itertools, MinMaxResult};
use std::io;
use std::io::Read;

fn main() {
    let stdin = io::stdin();
    let positions = stdin.lock().bytes().flatten().map_into::<char>().group_by(|c| c.is_digit(10));
    let mut positions = positions
        .into_iter()
        .map(|(_, v)| String::from_iter(v).parse::<i32>())
        .flatten()
        .collect_vec();

    if let MinMaxResult::MinMax(min, max) = positions.iter().minmax() {
        let (destination, fuel) = (*min..=*max)
            .into_iter()
            .map(|destination: i32| {
                let fuel: i32 = positions
                    .iter()
                    .map(|position| (*position - destination).abs())
                    .sum1()
                    .unwrap();
                (destination, fuel)
            })
            .min_by_key(|(_, fuel)| *fuel)
            .unwrap();

        println!("destination: {} fuel: {}", destination, fuel)
    };
}
