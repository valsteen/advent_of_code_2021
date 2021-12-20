use itertools::Itertools;
use std::borrow::Borrow;
use std::io;
use std::io::BufRead;

#[inline]
fn get_at(input: &[Vec<bool>], x: i32, y: i32, dx: i32, dy: i32, default: usize) -> usize {
    let x = x + dx;
    let y = y + dy;
    if x < 0 || y < 0 || y > input.len() as i32 - 1 || x > input[0].len() as i32 - 1 {
        default
    } else if input[y as usize][x as usize] {
        1
    } else {
        0
    }
}

#[inline]
fn zone(input: &[Vec<bool>], x: i32, y: i32, default: usize) -> usize {
    (-1..=1)
        .map(|dy| (-1..=1).map(move |dx| get_at(input, x, y, dx, dy, default)))
        .flatten()
        .fold(0, |acc, bit| (acc << 1) + bit)
}

#[inline]
fn enhance(input: &[Vec<bool>], enhancement: &[bool], x: i32, y: i32, default: usize) -> bool {
    enhancement[zone(input, x, y, default)]
}

fn next_default(default: usize, enhancement: &[bool]) -> usize {
    let default = if default == 1 { enhancement[(1 << 9) - 1] } else { enhancement[0] };
    if default {
        1
    } else {
        0
    }
}

fn enhance_image(image: &[Vec<bool>], enhancement: &[bool], default: usize) -> Vec<Vec<bool>> {
    (-1..=image[0].len() as i32)
        .map(|y| {
            (-1..=image.len() as i32).map(|x| enhance(image, enhancement, x, y, default)).collect()
        })
        .collect()
}

#[allow(dead_code)]
fn display(image: &[Vec<bool>]) -> String {
    (0..image[0].len())
        .map(|y| (0..image.len()).map(|x| if image[y][x] { "#" } else { "." }).collect::<String>())
        .join("\n")
}

fn main() {
    let stdin = io::stdin();

    let lines = stdin.lock().lines().flatten();
    let lines = lines.group_by(String::is_empty);
    let mut images = lines.borrow().into_iter().filter_map(|(is_empty, lines)| {
        if is_empty {
            None
        } else {
            Some(lines.map(|line| line.chars().map(|c| c == '#').collect_vec()).collect_vec())
        }
    });

    let enhancement = images.next().unwrap().into_iter().flatten().collect_vec();
    let image = images.next().unwrap();

    let mut default = 0;

    let new_image = (0..50).fold(image, |image, _| {
        let new_image = enhance_image(&image, &enhancement, default);
        default = next_default(default, &enhancement);
        new_image
    });

    let count = new_image.iter().flatten().filter(|&&b| b).count();
    println!("{}", count);
}
