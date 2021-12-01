use itertools::Itertools;

fn main() {

    let input = "199
200
208
210
200
207
240
269
260
263";
    let measurements : Vec<i32> = input.split("\n").map(|x| x.parse::<i32>().unwrap()).collect_vec();
    let windows = measurements.windows(3);


    let increases = windows.scan(None, |previous: &mut Option<i32>, current| {
        let sum = current.iter().sum();
        let is_increase = match previous {
            None => false,
            Some(value) => sum > *value
        };
        *previous = Some(sum);
        Some(is_increase)
    }).filter(|is_increase| *is_increase).count();

    println!("{}", increases);
}
