use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{stdin, BufRead};
use std::mem;
use std::rc::Rc;

struct Node {
    element: char,
    next: Option<Rc<RefCell<Node>>>,
}

impl Node {
    fn pair(&self) -> Option<String> {
        self.next.as_ref().map(|next| format!("{}{}", self.element, RefCell::borrow(next).element))
    }

    fn insert_after(&mut self, element: char) {
        let old = mem::take(&mut self.next);
        self.next = Some(Rc::new(RefCell::new(Node { element, next: old })));
    }
}

fn main() {
    let stdin = stdin();
    let mut lines = stdin.lock().lines().flatten();

    let line = lines.next().unwrap();
    let mut line = line.chars();

    let mut counts = HashMap::<char, usize>::new();
    let element = line.next().unwrap();
    let mut start = Some(Rc::new(RefCell::new(Node { element, next: None })));
    *counts.entry(element).or_default() += 1;

    let mut current = start.clone();

    for element in line {
        if let Some(node) = current.clone() {
            RefCell::borrow_mut(&node).insert_after(element);
            *counts.entry(element).or_default() += 1;
            current = RefCell::borrow(&node).next.clone();
        }
    }

    let rules: HashMap<_, _> = lines
        .skip(1)
        .map(|s| {
            let mut s = s.chars();
            (format!("{}{}", s.next().unwrap(), s.next().unwrap()), s.nth(4).unwrap())
        })
        .collect();

    for _ in 0..40 {
        current = start.clone();

        let mut first = true;

        loop {
            if let Some(node) = current.clone() {
                let pair = RefCell::borrow(&node).pair();
                if let Some(pair) = pair {
                    if let Some(&element) = rules.get(&pair) {
                        RefCell::borrow_mut(&node).insert_after(element);
                        *counts.entry(element).or_default() += 1;

                        if first {
                            start = current.clone();
                            first = false;
                        }
                        current = RefCell::borrow(&node).next.clone() // skip 1
                    }
                } else {
                    break;
                }

                current = RefCell::borrow(&current.unwrap()).next.clone()
            }
        }
    }

    let mut min = usize::MAX;
    let mut max = 0;

    for &count in counts.values() {
        min = min.min(count);
        max = max.max(count);
    }
    println!("{}-{}={}", max, min, max - min);
}
