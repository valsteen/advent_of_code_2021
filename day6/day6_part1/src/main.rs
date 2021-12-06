use std::io;
use std::io::BufRead;

fn main() {
    let stdin = io::stdin();
    let line = stdin.lock().lines().flatten().next().unwrap();
    let mut ages: Vec<usize> = line.split(',').map(str::parse).flatten().collect();

    for _ in 0..80 {
        let mut births = vec![];
        for age in ages.iter_mut() {
            match age {
                0 => {
                    births.push(8);
                    *age = 6;
                }
                _ => {
                    *age -= 1;
                }
            }
        }
        ages.extend(births);
    }

    println!("{}", ages.len());
}
