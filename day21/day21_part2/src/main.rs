use itertools::Itertools;
use num::bigint::BigInt;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::io;
use std::io::BufRead;

#[derive(Clone, Debug)]
struct Step {
    player_1_position: u8,
    player_2_position: u8,
    player_1_score: u8,
    player_2_score: u8,
    next_player: u8,
    previous: Option<Box<Step>>,
    factor: BigInt,
}

impl Hash for Step {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (
            self.player_1_position,
            self.player_2_position,
            self.player_1_score,
            self.player_2_score,
            self.next_player,
        )
            .hash(state)
    }
}

impl PartialEq<Self> for Step {
    fn eq(&self, other: &Self) -> bool {
        (
            self.player_1_position,
            self.player_2_position,
            self.player_1_score,
            self.player_2_score,
            self.next_player,
        )
            .eq(&(
                other.player_1_position,
                other.player_2_position,
                other.player_1_score,
                other.player_2_score,
                other.next_player,
            ))
    }
}

impl Eq for Step {}

enum StepResult {
    Win(Step, u8),
    Next(Vec<Step>),
}

const GOAL: u8 = 21;

impl Step {
    fn next(self: Step) -> StepResult {
        if self.player_1_score >= GOAL || self.player_2_score >= GOAL {
            StepResult::Win(self.clone(), (self.next_player + 1) % 2)
        } else {
            let next_steps = [(3, 1), (4, 3), (5, 6), (6, 7), (7, 6), (8, 3), (9, 1)]
                .into_iter()
                .map(|(increment, occurrences)| {
                    if self.next_player == 0 {
                        let position = (self.player_1_position + increment - 1) % 10 + 1;
                        Step {
                            player_1_position: position,
                            player_2_position: self.player_2_position,
                            player_1_score: self.player_1_score + position,
                            player_2_score: self.player_2_score,
                            next_player: 1,
                            previous: Some(Box::new(self.clone())),
                            factor: BigInt::from(occurrences),
                        }
                    } else {
                        let position = (self.player_2_position + increment - 1) % 10 + 1;
                        Step {
                            player_1_position: self.player_1_position,
                            player_2_position: position,
                            player_1_score: self.player_1_score,
                            player_2_score: self.player_2_score + position,
                            next_player: 0,
                            previous: Some(Box::new(self.clone())),
                            factor: BigInt::from(occurrences),
                        }
                    }
                })
                .collect();

            StepResult::Next(next_steps)
        }
    }
}

fn main() {
    let stdin = io::stdin();
    let players = stdin
        .lock()
        .lines()
        .flatten()
        .map(|s| s.rsplit_once(' ').unwrap().1.parse())
        .flatten()
        .collect_vec();

    let start = Step {
        player_1_position: players[0],
        player_2_position: players[1],
        player_1_score: 0,
        player_2_score: 0,
        next_player: 0,
        previous: None,
        factor: BigInt::from(1),
    };

    let mut knownsteps = HashMap::<Step, Vec<(Step, BigInt)>>::new();
    let mut knownoccurrences = HashMap::<Step, BigInt>::new();
    knownoccurrences.insert(start.clone(), BigInt::from(1));
    let mut wins = HashMap::new();
    let mut steps = vec![start];

    while let Some(step) = steps.pop() {
        if let Some(occurrences) = knownsteps.get_mut(&step) {
            occurrences.push((*step.previous.unwrap(), step.factor));
            continue;
        }
        if let Some(previous) = &step.previous {
            knownsteps.insert(step.clone(), vec![(*previous.clone(), step.factor.clone())]);
        }

        match step.next() {
            StepResult::Next(next_steps) => steps.extend(next_steps),

            StepResult::Win(step, player) => {
                wins.insert(step, player);
            }
        }
    }

    let (mut wins_1, mut wins_2) = (BigInt::from(0), BigInt::from(0));

    while !knownsteps.is_empty() {
        let mut remove = vec![];
        for (step, previous) in knownsteps.iter() {
            if previous.iter().all(|(step, _)| knownoccurrences.contains_key(step)) {
                let occurrences: BigInt = previous
                    .iter()
                    .map(|(step, occurrences)| {
                        knownoccurrences.get(step).unwrap().clone() * occurrences
                    })
                    .sum();
                knownoccurrences.insert(step.clone(), occurrences.clone());
                remove.push(step.clone());

                if let Some(&player) = wins.get(step) {
                    if player == 0 {
                        wins_1 += occurrences;
                    } else {
                        wins_2 += occurrences;
                    }
                }
            }
        }

        for step in remove.drain(..) {
            knownsteps.remove(&step);
        }
    }

    println!("{}", wins_1.max(wins_2));
}
