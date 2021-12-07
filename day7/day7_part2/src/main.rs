use itertools::{Itertools, MinMaxResult};
use std::io;
use std::io::Read;

fn main() {
    let stdin = io::stdin();
    let positions = stdin.lock().bytes().flatten().map_into::<char>().group_by(|c| c.is_digit(10));
    let positions =
        positions.into_iter().map(|(_, v)| String::from_iter(v).parse()).flatten().collect_vec();

    if let MinMaxResult::MinMax(min, max) = positions.iter().minmax() {
        let (destination, fuel) = (*min..=*max)
            .map(|destination: i64| {
                let fuel: i64 = positions
                    .iter()
                    .map(|position| {
                        let diff = (*position - destination).abs();
                        diff * (diff + 1) / 2
                    })
                    .sum();
                (destination, fuel)
            })
            .min_by_key(|(_, fuel)| *fuel)
            .unwrap();

        println!("destination: {} fuel: {}", destination, fuel)
    };
}
