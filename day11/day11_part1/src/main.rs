use std::io;
use std::io::Read;

fn apply_step_to_neighbours<T: FnMut(usize, usize)>(x: usize, y: usize, mut f: T) {
    if x > 0 {
        f(x - 1, y);

        if y > 0 {
            f(x - 1, y - 1)
        }

        if y < 9 {
            f(x - 1, y + 1)
        }
    }

    if y > 0 {
        f(x, y - 1)
    }

    if y < 9 {
        f(x, y + 1)
    }

    if x < 9 {
        f(x + 1, y);

        if y > 0 {
            f(x + 1, y - 1)
        }

        if y < 9 {
            f(x + 1, y + 1)
        }
    }
}

fn step(grid: &mut [[u8; 10]; 10]) -> usize {
    let mut flashes_to_process = vec![];

    for (x, line) in grid.iter_mut().enumerate() {
        for (y, item) in line.iter_mut().enumerate() {
            *item += 1;
            if *item == 10 {
                flashes_to_process.push((x, y));
            }
        }
    }

    let mut flashes = 0;

    while let Some((x, y)) = flashes_to_process.pop() {
        flashes += 1;
        apply_step_to_neighbours(x, y, |x, y| {
            grid[x][y] += 1;
            if grid[x][y] == 10 {
                flashes_to_process.push((x, y));
            }
        });
    }

    for (_, line) in grid.iter_mut().enumerate() {
        for (_, item) in line.iter_mut().enumerate() {
            if *item > 9 {
                *item = 0;
            }
        }
    }

    flashes
}

fn main() {
    let mut grid = [[0u8; 10]; 10];

    let stdin = io::stdin();
    for (position, value) in stdin.lock().bytes().flatten().filter(u8::is_ascii_digit).enumerate() {
        grid[position % 10][position / 10] = value - b'0';
    }
    let mut flashes = 0;
    for _ in 0..100 {
        flashes += step(&mut grid);
    }

    println!("{}", flashes);
}
