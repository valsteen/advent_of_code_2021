use std::io;
use std::io::Read;

fn main() {
    let stdin = io::stdin();
    let lines: String = stdin.lock().bytes().flatten().map(char::from).collect();
    let lines: Vec<Vec<String>> = lines
        .replace("|\n", "")
        .replace('|', "")
        .split('\n')
        .map(|line| line.split(' ').map(String::from).collect())
        .filter(|line: &Vec<String>| line.len() > 1)
        .collect();

    let count: usize = lines
        .iter()
        .map(|line| {
            line.split_at(10).1.iter().filter(|item| matches!(item.len(), 2 | 4 | 3 | 7)).count()
        })
        .sum();
    println!("{:?}", count);
}
