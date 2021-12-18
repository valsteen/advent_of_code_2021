use itertools::Itertools;
use rayon::prelude::*;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::io;
use std::io::BufRead;
use std::ops::{Deref, DerefMut};
use std::rc::{Rc, Weak};

enum Node {
    Number(i32, Weak<RefCell<Node>>),
    Pair(Vec<Rc<RefCell<Node>>>, Weak<RefCell<Node>>),
}

impl Node {
    fn add_node(parent: &Rc<RefCell<Self>>, node: Rc<RefCell<Self>>) -> Rc<RefCell<Self>> {
        if let Node::Pair(parts, ..) = RefCell::borrow_mut(parent).deref_mut() {
            parts.push(node.clone());
            node
        } else {
            unreachable!()
        }
    }

    fn add_number(parent: &Rc<RefCell<Self>>, n: i32) -> Rc<RefCell<Self>> {
        let n = Rc::new(RefCell::new(Self::Number(n, Rc::downgrade(parent))));
        Self::add_node(parent, n)
    }

    fn add_empty_node(parent: Option<&Rc<RefCell<Self>>>) -> Rc<RefCell<Self>> {
        let parent_weak =
            if let Some(parent) = parent { Rc::downgrade(parent) } else { Weak::new() };
        let node = Rc::new(RefCell::new(Self::Pair(vec![], parent_weak)));
        if let Some(parent) = parent {
            Self::add_node(parent, node)
        } else {
            node
        }
    }

    fn find_number_downwards(node: &Rc<RefCell<Self>>, direction: Direction) -> Rc<RefCell<Self>> {
        match RefCell::borrow(node).deref() {
            Node::Number(_, _) => node.clone(),
            Node::Pair(parts, _) => match direction {
                Direction::Left => Node::find_number_downwards(parts.first().unwrap(), direction),
                Direction::Right => Node::find_number_downwards(parts.last().unwrap(), direction),
            },
        }
    }

    fn neighbour_number(
        node: &Rc<RefCell<Self>>,
        direction: Direction,
    ) -> Option<Rc<RefCell<Self>>> {
        let node_inner = RefCell::borrow(node);
        let parent = match node_inner.deref() {
            Node::Number(_, parent) => parent,
            Node::Pair(_, parent) => parent,
        };
        if let Some(parent) = parent.upgrade() {
            match RefCell::borrow(&parent).deref() {
                Node::Number(_, _) => unreachable!(),
                Node::Pair(parts, _) => {
                    let neighbour = match direction {
                        Direction::Left => parts.first().unwrap(),
                        Direction::Right => parts.last().unwrap(),
                    };
                    if Rc::ptr_eq(neighbour, node) {
                        Node::neighbour_number(&parent, direction)
                    } else {
                        let node_inner = RefCell::borrow(neighbour);
                        let node = node_inner.deref();
                        match node {
                            Node::Number(_, _) => Some(neighbour.clone()),
                            Node::Pair(_, _) => {
                                Some(Node::find_number_downwards(neighbour, direction.opposite()))
                            }
                        }
                    }
                }
            }
        } else {
            None
        }
    }

    fn parent(&self) -> Weak<RefCell<Node>> {
        match self {
            Node::Number(_, parent) => parent.clone(),
            Node::Pair(_, parent) => parent.clone(),
        }
    }

    fn set_parent(node: &Rc<RefCell<Node>>, parent: &Weak<RefCell<Node>>) -> Rc<RefCell<Node>> {
        let node_inner = RefCell::borrow(node);
        match node_inner.deref() {
            &Node::Number(n, _) => Rc::new(RefCell::new(Node::Number(n, parent.clone()))),
            Node::Pair(pair, _) => {
                let new_node = Rc::new(RefCell::new(Self::Pair(vec![], parent.clone())));
                for node in pair {
                    let node = Node::set_parent(node, &Rc::downgrade(&new_node));
                    Node::add_node(&new_node, node);
                }
                new_node
            }
        }
    }

    fn number(&self) -> i32 {
        match self {
            &Node::Number(n, _) => n,
            Node::Pair(..) => unreachable!(),
        }
    }

    fn magnitude(&self) -> usize {
        match self {
            &Node::Number(n, _) => n as usize,
            Node::Pair(parts, _) => {
                let first = parts.first().unwrap();
                let first_inner = RefCell::borrow(first);
                let last = parts.last().unwrap();
                let last_inner = RefCell::borrow(last);

                first_inner.magnitude() * 3 + last_inner.magnitude() * 2
            }
        }
    }

    fn from_string(line: String) -> Rc<RefCell<Node>> {
        let groups = line.chars().group_by(char::is_ascii_digit);
        let seed = Node::add_empty_node(None);
        groups.borrow().into_iter().fold(
            seed.clone(),
            |mut acc: Rc<RefCell<Node>>, (is_number, group)| {
                if is_number {
                    let number = String::from_iter(group).parse().unwrap();
                    Node::add_number(&acc, number);
                } else {
                    for c in group {
                        match c {
                            '[' => acc = Node::add_empty_node(Some(&acc)),
                            ']' => {
                                let parent = match RefCell::borrow(&acc).deref() {
                                    Node::Number(_, _) => unreachable!(),
                                    Node::Pair(_, parent) => parent.upgrade().unwrap(),
                                };
                                acc = parent;
                            }
                            _ => {}
                        }
                    }
                }
                acc
            },
        );

        let seed = if let Node::Pair(result, _) = RefCell::borrow_mut(&seed).deref_mut() {
            result.pop().unwrap()
        } else {
            unreachable!();
        };
        seed
    }
}

impl Debug for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Node::Number(n, _) => Debug::fmt(n, f),
            Node::Pair(parts, _) => {
                let out = parts
                    .iter()
                    .map(|part| {
                        let node_inner = RefCell::borrow(part);
                        format!("{:?}", node_inner.deref())
                    })
                    .join(",");
                f.write_str("[")?;
                f.write_str(&out)?;
                f.write_str("]")
            }
        }
    }
}

enum Direction {
    Left,
    Right,
}

impl Direction {
    fn opposite(self) -> Self {
        match self {
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

enum Action {
    Explode(Rc<RefCell<Node>>),
    Split(Rc<RefCell<Node>>),
    Noop,
}

impl Debug for Action {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::Explode(node) => f.write_str(&format!("Explode ({:?})", node)),
            Action::Split(node) => f.write_str(&format!("Split ({:?})", node)),
            Action::Noop => f.write_str("Noop"),
        }
    }
}

impl Action {
    fn execute(self) {
        match self {
            Action::Noop => {}

            Action::Explode(node) => {
                let (left_value, right_value) = {
                    let node_inner = RefCell::borrow_mut(&node);

                    match node_inner.deref() {
                        Node::Number(_, _) => unreachable!(),
                        Node::Pair(nodes, _) => {
                            let left_inner = RefCell::borrow_mut(nodes.first().unwrap());
                            let right_inner = RefCell::borrow_mut(nodes.last().unwrap());
                            match (left_inner.deref(), right_inner.deref()) {
                                (&Node::Number(left, _), &Node::Number(right, _)) => (left, right),
                                _ => unreachable!(),
                            }
                        }
                    }
                };

                for (direction, value) in
                    [(Direction::Left, left_value), (Direction::Right, right_value)]
                {
                    if let Some(neighbour) = Node::neighbour_number(&node, direction) {
                        let mut neighbour_inner = RefCell::borrow_mut(&neighbour);
                        match neighbour_inner.deref_mut() {
                            Node::Number(neighbour_value, _) => {
                                *neighbour_value += value;
                            }
                            Node::Pair(_, _) => unreachable!(),
                        }
                    }
                }

                let mut node_inner = RefCell::borrow_mut(&node);
                let node = node_inner.deref_mut();
                *node = Node::Number(0, node.parent());
            }
            Action::Split(node) => {
                let (new_node, num) = {
                    let node_inner = RefCell::borrow_mut(&node);
                    let num = node_inner.number();
                    (Node::Pair(vec![], node_inner.parent()), num)
                };
                RefCell::replace(&node, new_node);

                Node::add_number(&node, (num as f32 / 2f32).floor() as i32);
                Node::add_number(&node, (num as f32 / 2f32).ceil() as i32);
            }
        }
    }
}

fn reduce(node: &Rc<RefCell<Node>>) {
    loop {
        let action = traverse(node, 0);
        match action {
            Action::Explode(_) | Action::Split(_) => action.execute(),
            Action::Noop => break,
        }
    }
}

fn traverse(node: &Rc<RefCell<Node>>, depth: usize) -> Action {
    let node_inner = RefCell::borrow_mut(node);
    let mut action = match node_inner.deref() {
        &Node::Number(n, _) => {
            if n >= 10 {
                Action::Split(node.clone())
            } else {
                Action::Noop
            }
        }
        Node::Pair(_, _) => {
            if depth == 4 {
                Action::Explode(node.clone())
            } else {
                Action::Noop
            }
        }
    };

    match action {
        Action::Explode(_) | Action::Split(_) => action,
        Action::Noop => match node_inner.deref() {
            Node::Number(_, _) => Action::Noop,
            Node::Pair(parts, _) => {
                for part in parts {
                    let new_action = traverse(part, depth + 1);
                    match new_action {
                        Action::Explode(_) => return new_action,
                        Action::Split(_) => match action {
                            Action::Explode(_) => unreachable!(),
                            Action::Split(_) => {}
                            Action::Noop => action = new_action,
                        },
                        Action::Noop => {}
                    }
                }
                action
            }
        },
    }
}

fn main() {
    let stdin = io::stdin();

    let lines = stdin.lock().lines().flatten().map(Node::from_string).collect_vec();

    for line in lines.iter() {
        reduce(line)
    }

    let best = lines
        .iter()
        .map(|node| {
            let node_inner = RefCell::borrow(node);
            format!("{:?}", node_inner.deref())
        })
        .permutations(2)
        .collect_vec()
        .into_par_iter()
        .map(|combination| {
            let root = Rc::new(RefCell::new(Node::Pair(vec![], Weak::new())));
            for node in combination {
                let node = Node::from_string(node);
                let node = Node::set_parent(&node, &Rc::downgrade(&root));
                Node::add_node(&root, node);
            }

            reduce(&root);
            let root_inner = RefCell::borrow(&root);
            root_inner.magnitude()
        })
        .max()
        .unwrap();

    println!("{}", best);
}
