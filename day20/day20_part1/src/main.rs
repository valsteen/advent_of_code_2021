use itertools::Itertools;
use rayon::prelude::*;
use std::borrow::Borrow;
use std::io;
use std::io::BufRead;

#[inline]
fn get_at(input: &[Vec<u8>], x: i32, y: i32, default: u8) -> u8 {
    if x < 0 || y < 0 || y > input.len() as i32 - 1 || x > input[0].len() as i32 - 1 {
        default
    } else {
        input[y as usize][x as usize]
    }
}

#[inline]
fn zone(input: &[Vec<u8>], x: i32, y: i32, default: u8) -> usize {
    (-1..=1)
        .map(|dy| (-1..=1).map(move |dx| get_at(input, x + dx, y + dy, default)))
        .flatten()
        .fold(0, |acc, bit| (acc << 1) + bit as usize)
}

fn next_default(default: u8, enhancement: &[u8]) -> u8 {
    if default == 1 {
        enhancement[(1 << 9) - 1]
    } else {
        enhancement[0]
    }
}

fn enhance_image(image: &[Vec<u8>], enhancement: &[u8], default: u8) -> Vec<Vec<u8>> {
    (-1..=image[0].len() as i32)
        .into_par_iter()
        .map(|y| {
            (-1..=image.len() as i32).map(|x| enhancement[zone(image, x, y, default)]).collect()
        })
        .collect()
}

#[allow(dead_code)]
fn display(image: &[Vec<u8>]) -> String {
    (0..image[0].len())
        .map(|y| {
            (0..image.len()).map(|x| if image[y][x] == 1 { "#" } else { "." }).collect::<String>()
        })
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
            Some(
                lines
                    .map(|line| line.chars().map(|c| if c == '#' { 1 } else { 0 }).collect_vec())
                    .collect_vec(),
            )
        }
    });

    let enhancement = images.next().unwrap().into_iter().flatten().collect_vec();
    let image = images.next().unwrap();

    let mut default = 0;

    let new_image = (0..2).fold(image, |image, _| {
        let new_image = enhance_image(&image, &enhancement, default);
        default = next_default(default, &enhancement);
        new_image
    });

    let count = new_image.par_iter().flatten().map(|&v| v as usize).sum::<usize>();
    println!("{}", count);
}
