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
    let measurements = input.split("\n").map(|x| x.parse::<i32>().unwrap());

    let increases = measurements.scan(None, |previous: &mut Option<i32>, current| {
        let is_increase = match previous {
            None => false,
            Some(value) => current > *value
        };
        *previous = Some(current);
        Some(is_increase)
    }).filter(|is_increase| *is_increase).count();

    println!("{}", increases);
}
