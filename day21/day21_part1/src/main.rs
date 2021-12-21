use std::io;
use std::io::BufRead;
use std::str::FromStr;

#[derive(Debug)]
struct Dice {
    value: usize,
    rolls: usize,
}

impl Dice {
    fn new() -> Self {
        Self { value: 99, rolls: 0 }
    }
    fn roll(&mut self) -> usize {
        self.value = (self.value + 1) % 100;
        self.rolls += 1;
        self.value + 1
    }
}

#[derive(Debug)]
struct Player {
    position: usize,
    score: usize,
}

impl Player {
    fn step(&mut self, dice: &mut Dice) {
        let action = (0..=2).fold(0, |acc, _| acc + dice.roll());
        self.position = (self.position - 1 + action) % 10 + 1;
        self.score += self.position
    }
}

impl FromStr for Player {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Player { position: s.parse().map_err(|_| ())?, score: 0 })
    }
}

fn main() {
    let stdin = io::stdin();
    let mut players = stdin
        .lock()
        .lines()
        .flatten()
        .map(|s| s.rsplit_once(' ').unwrap().1.parse())
        .flatten()
        .collect::<Vec<Player>>();

    let mut dice = Dice::new();
    'main: loop {
        for p in players.iter_mut() {
            p.step(&mut dice);
            if p.score >= 1000 {
                break 'main;
            }
        }
    }
    let loser = players.into_iter().find(|p| p.score < 1000).unwrap();
    println!("{}", loser.score * dice.rolls);
}
