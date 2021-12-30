#![cfg_attr(feature = "pedantic", warn(clippy::pedantic))]
#![warn(clippy::use_self)]
#![warn(clippy::map_flatten)]
#![warn(clippy::map_unwrap_or)]
#![warn(deprecated_in_future)]
#![warn(future_incompatible)]
#![warn(noop_method_call)]
#![warn(missing_debug_implementations)]
#![warn(rust_2018_compatibility)]
#![warn(rust_2021_compatibility)]
#![warn(rust_2018_idioms)]
#![warn(unused)]
#![deny(warnings)]

use itertools::Itertools;
use lru::LruCache;
use std::cell::{Ref, RefCell, RefMut};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::io;
use std::io::BufRead;
use std::rc::Rc;
use std::str::FromStr;

#[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
enum Variable {
    X,
    Y,
    Z,
    W,
}

impl Display for Variable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{:?}", self))
    }
}

impl FromStr for Variable {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let result = match s {
            "x" => Self::X,
            "y" => Self::Y,
            "z" => Self::Z,
            "w" => Self::W,
            _ => return Err(()),
        };
        Ok(result)
    }
}

#[derive(Debug)]
enum VariableOrNumber {
    Variable(Variable),
    Number(i64),
}

#[derive(Debug, Clone)]
struct ValueRef {
    rc_value: Rc<ValueContainer>,
}

impl PartialOrd<Self> for ValueRef {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for ValueRef {}

impl Ord for ValueRef {
    fn cmp(&self, other: &Self) -> Ordering {
        let op1 = OP_ORDER.iter().position(|o| self.op_str() == *o).unwrap();
        let op2 = OP_ORDER.iter().position(|o| other.op_str() == *o).unwrap();

        op1.cmp(&op2).then_with(|| match (&*self.ref_value(), &*other.ref_value()) {
            (Value::Number(n1), Value::Number(n2)) => n1.cmp(n2),
            (Value::Input(n1), Value::Input(n2)) => n1.cmp(n2),
            _ => self
                .children()
                .cmp(&other.children())
                .then_with(|| self.equalities().cmp(&other.equalities())),
        })
    }
}

impl PartialEq for ValueRef {
    fn eq(&self, other: &Self) -> bool {
        let res = match (&*self.ref_value(), &*other.ref_value()) {
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::Input(a), Value::Input(b)) => a == b,
            _ => Rc::ptr_eq(&self.rc_value, &other.rc_value),
        };
        res && self.rc_value.solved.borrow().0 == other.rc_value.solved.borrow().0
    }
}

impl Hash for ValueRef {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::hash(self.op_str(), state);
        self.rc_value.solved.borrow().0.hash(state);
        for child in self.children() {
            std::ptr::hash(Rc::as_ptr(&child.rc_value), state);
        }

        match &*self.ref_value() {
            Value::Number(n) => n.hash(state),
            Value::Input(n) => n.hash(state),
            _ => {}
        }
    }
}

const OP_ORDER: [&str; 7] = ["Number", "Input", "Add", "Mul", "Div", "Mod", "Eql"];

#[derive(Debug)]
struct ValueContainer {
    inner: RefCell<Value>,
    solved: RefCell<Equalities>,
}

#[derive(Debug, PartialEq, Eq, Clone, Ord, PartialOrd, Hash)]
enum Value {
    Number(i64),
    Input(u8),
    Add(ValueRef, ValueRef),
    Mul(ValueRef, ValueRef),
    Div(ValueRef, ValueRef),
    Mod(ValueRef, ValueRef),
    Eql(ValueRef, ValueRef, Option<u8>),
}

impl Display for ValueRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let out = match &*self.ref_value() {
            Value::Number(n) => n.to_string(),
            Value::Input(n) => {
                format!("Input({})", n)
            }
            Value::Add(a, b)
            | Value::Mul(a, b)
            | Value::Div(a, b)
            | Value::Mod(a, b)
            | Value::Eql(a, b, _) => {
                format!("{}({}, {})", self.op_str(), a, b)
            }
        };
        f.write_str(&out)
    }
}

type ResolutionCache = LruCache<ValueRef, HashMap<[Option<i64>; 14], ValueRef>>;

struct Factory {
    cache: RefCell<LruCache<ValueRef, ValueRef>>,
    input_dependencies: RefCell<LruCache<ValueRef, [bool; 14]>>,
    inputs_resolves: RefCell<ResolutionCache>,
    deductions: RefCell<LruCache<ValueRef, Deduction>>,
}

impl Factory {
    fn get_cached(&self, value: &ValueRef) -> Option<ValueRef> {
        let cache = &mut *self.cache.borrow_mut();
        let cached = cache.get(value);
        cached.cloned()
    }

    fn set_deduction(
        &self,
        value_ref: &ValueRef
    ) -> Deduction {
        if let Some(deduction) = self.deductions.borrow_mut().get(value_ref) {
            return deduction.clone();
        }

        let deduction = match &*value_ref.ref_value() {
            Value::Number(n) => Deduction::In { min: *n, max: *n },
            Value::Input(_) => Deduction::In { min: 1, max: 9 },
            _ => {
                let children = value_ref.children();
                let deduction_a = self.set_deduction(&children[0]);
                let deduction_b = self.set_deduction(&children[1]);

                if deduction_a == Deduction::Impossible || deduction_b == Deduction::Impossible {
                    Deduction::Impossible
                } else if let (
                    Deduction::In {
                        min: min_a,
                        max: max_a,
                    },
                    Deduction::In {
                        min: mut min_b,
                        max: mut max_b,
                    },
                ) = (deduction_a, deduction_b)
                {
                    match &*value_ref.ref_value() {
                        Value::Add(_, _) => {
                            Deduction::In {
                                min: min_a + min_b,
                                max: max_a + max_b,
                            }
                        }
                        Value::Mul(_, _) => {
                            if min_a == max_a || min_b == max_b {
                                Deduction::In {
                                    min: (min_b * min_a).min(max_b * max_a),
                                    max: (min_b * min_a).max(max_b * max_a),
                                }
                            } else {
                                let mut results =
                                    [min_a * min_b, min_a * max_b, max_a * min_b, max_a * max_b];
                                results.sort_unstable();
                                Deduction::In {
                                    min: results[0],
                                    max: results[3],
                                }
                            }
                        }
                        Value::Div(_, _) => {
                            if max_a < min_b {
                                Deduction::In { min: 0, max: 0 }
                            } else {
                                if min_b < max_b {
                                    if min_b == 0 {
                                        min_b += 1
                                    } else if max_b == 0 {
                                        max_b -= 1
                                    }
                                }
                                let results = [
                                    min_a.checked_div(min_b),
                                    min_a.checked_div(max_b),
                                    max_a.checked_div(min_b),
                                    max_a.checked_div(max_b),
                                ];
                                results.iter().flatten().minmax().into_option().map_or(
                                    Deduction::Impossible,
                                    |(&min, &max)| Deduction::In {
                                        min,
                                        max,
                                    },
                                )
                            }
                        }
                        Value::Mod(_, _) => {
                            if (min_a < 0 && max_a < 0) || (min_b <= 0 && max_b <= 0) {
                                Deduction::Impossible
                            } else if max_a < min_b {
                                Deduction::In {
                                    min: min_a.max(0),
                                    max: max_a
                                }
                            } else {
                                Deduction::In { min: 0, max: max_b - 1 }
                            }
                        }

                        Value::Eql(_, _, _) => {
                            if min_a == max_a && min_a == min_b && min_a == max_b {
                                Deduction::In { min: 1, max: 1 }
                            } else if max_a < min_b || min_a > max_b {
                                Deduction::In { min: 0, max: 0 }
                            } else {
                                Deduction::In { min: 0, max: 1 }
                            }
                        }
                        _ => unreachable!(),
                    }
                } else {
                    Deduction::Unknown
                }
            }
        };

        self.deductions.borrow_mut().put(value_ref.clone(), deduction.clone());
        deduction
    }

    fn set_input_dependencies(&self, value_ref: &ValueRef) -> [bool; 14] {
        if let Some(dependencies) = self.input_dependencies.borrow_mut().get(value_ref) {
            return *dependencies;
        }

        let ref_dependencies = {
            match &*value_ref.ref_value() {
                Value::Input(n) => {
                    let mut inputs = [false; 14];
                    inputs[*n as usize] = true;
                    inputs
                }
                Value::Number(_) => [false; 14],
                Value::Add(a, b)
                | Value::Mul(a, b)
                | Value::Div(a, b)
                | Value::Mod(a, b)
                | Value::Eql(a, b, _) => {
                    let mut inputs = self.set_input_dependencies(a);
                    let b_dependencies = self.set_input_dependencies(b);

                    for (k, v) in b_dependencies.iter().enumerate() {
                        inputs[k] |= *v
                    }
                    inputs
                }
            }
        };
        self.input_dependencies.borrow_mut().put(value_ref.clone(), ref_dependencies);
        ref_dependencies
    }

    fn new_is_reducible(
        &self,
        value: Value,
        solved: Option<Equalities>,
    ) -> Result<(ValueRef, bool), &'static str> {
        let solved = solved.unwrap_or(Equalities([None; 14]));

        let mut result = ValueRef {
            rc_value: Rc::new(ValueContainer {
                inner: RefCell::new(value),
                solved: RefCell::new(solved),
            }),
        };

        if let Some(target_value) = self.get_cached(&result) {
            return Ok((target_value, false));
        }
        let reduced = result.reduce(self)?;

        if let Some(value) = self.get_cached(&result) {
            return Ok((value, reduced));
        }
        self.set_input_dependencies(&result);
        self.set_deduction(&result);
        self.inputs_resolves.borrow_mut().put(result.clone(), HashMap::new());
        self.cache.borrow_mut().put(result.clone(), result.clone());
        Ok((result, reduced))
    }

    fn try_new(&self, value: Value, solved: Option<Equalities>) -> Result<ValueRef, &'static str> {
        self.new_is_reducible(value, solved).map(|(value, _)| value)
    }
}

impl ValueRef {
    fn equalities(&self) -> Equalities {
        self.rc_value.solved.borrow().clone()
    }

    fn op_str(&self) -> &'static str {
        match &*self.ref_value() {
            Value::Number(_) => "Number",
            Value::Input(_) => "Input",
            Value::Add(_, _) => "Add",
            Value::Mul(_, _) => "Mul",
            Value::Div(_, _) => "Div",
            Value::Mod(_, _) => "Mod",
            Value::Eql(_, _, _) => "Eql",
        }
    }
    fn value_for_inputs(
        &self,
        inputs: &[Option<i64>; 14],
        factory: &Factory,
    ) -> Result<Self, &'static str> {
        let mut key = *inputs;
        let input_dependencies = factory.set_input_dependencies(self);

        for (i, v) in input_dependencies.iter().enumerate() {
            if !v {
                key[i] = None
            }
        }

        if let Some(value) =
            factory.inputs_resolves.borrow_mut().get(self).and_then(|e| e.get(&key))
        {
            return Ok(value.clone());
        };

        let value = self.solve(factory, &key)?;
        {
            // does not change a thing
            let eq = { value.rc_value.solved.borrow().clone() };
            let eq = eq.add(&value.equalities());
            value.rc_value.solved.replace(eq);
        }

        let mut inputs_resolves = factory.inputs_resolves.borrow_mut();

        let entry = if let Some(entry) = inputs_resolves.get_mut(self) {
            entry
        } else {
            inputs_resolves.put(self.clone(), HashMap::new());
            inputs_resolves.get_mut(self).unwrap()
        };
        entry.insert(key, value.clone());
        Ok(value)
    }

    fn solve(&self, factory: &Factory, inputs: &[Option<i64>; 14]) -> Result<Self, &'static str> {
        let (a, b, eql) = match &*self.ref_value() {
            Value::Number(_) => return Ok(self.clone()),
            Value::Input(n) => {
                return if let Some(value) = inputs[*n as usize] {
                    factory.try_new(Value::Number(value), None)
                } else {
                    Ok(self.clone())
                };
            }
            Value::Add(a, b)
            | Value::Mul(a, b)
            | Value::Div(a, b)
            | Value::Mod(a, b)
            | Value::Eql(a, b, _) => {
                let found = if let Value::Eql(_, _, Some(idx)) = &*self.ref_value() {
                    Some(*idx)
                } else {
                    None
                };
                let new_a = a.value_for_inputs(inputs, factory)?;
                let new_b = b.value_for_inputs(inputs, factory)?;
                let equalities =
                    self.equalities().add(&new_a.equalities()).add(&new_b.equalities()); // ...
                let result = if let (&Value::Number(n_a), &Value::Number(n_b)) =
                    (&*new_a.ref_value(), &*new_b.ref_value())
                {
                    (n_a, n_b, equalities)
                } else {
                    if &new_a == a && &new_b == b {
                        return Ok(self.clone());
                    }

                    let new_value = match &*self.ref_value() {
                        Value::Number(_) | Value::Input(_) => unreachable!(),
                        Value::Add(_, _) => Value::Add(new_a.clone(), new_b.clone()),
                        Value::Mul(_, _) => Value::Mul(new_a.clone(), new_b.clone()),
                        Value::Div(_, _) => Value::Div(new_a.clone(), new_b.clone()),
                        Value::Mod(_, _) => Value::Mod(new_a.clone(), new_b.clone()),
                        Value::Eql(_, _, _) => Value::Eql(new_a.clone(), new_b.clone(), found),
                    };

                    return factory.try_new(new_value, Some(equalities));
                };
                result
            }
        };

        let mut found = self.equalities().add(&eql);
        let result = match &*self.ref_value() {
            Value::Input(_) | Value::Number(_) => unreachable!(),
            Value::Add(_, _) => a + b,
            Value::Mul(_, _) => a * b,
            Value::Div(_, _) => a.checked_div(b).ok_or("Division by zero")?,
            Value::Mod(_, _) => {
                if a < 0 || b <= 0 {
                    return Err("Invalid modulo");
                }
                a % b
            }
            Value::Eql(_, _, is_var) => {
                if a == b {
                    if let Some(v) = is_var {
                        found.win(*v)
                    }
                    1
                } else {
                    if let Some(v) = is_var {
                        found.fail(*v)
                    }
                    0
                }
            }
        };
        factory.try_new(Value::Number(result), Some(found))
    }

    fn ref_value(&self) -> Ref<'_, Value> {
        RefCell::borrow(&self.rc_value.inner)
    }

    fn clone_value(&self) -> Value {
        (&*self.ref_value()).clone()
    }

    fn ref_mut_value(&self) -> RefMut<'_, Value> {
        RefCell::borrow_mut(&self.rc_value.inner)
    }

    fn reorder<'a>(values: (&'a Self, &'a Self)) -> (&'a Self, &'a Self, bool) {
        if values.0 > values.1 {
            (values.1, values.0, true)
        } else {
            (values.0, values.1, false)
        }
    }

    fn reorder_4(values: [&Self; 4]) -> ([&Self; 4], bool) {
        let mut result = values;
        result.sort();
        (result, values != result)
    }

    fn reorder_3(values: [&Self; 3]) -> ([&Self; 3], bool) {
        let mut result = values;
        result.sort();
        (result, values != result)
    }

    fn children(&self) -> Vec<Self> {
        match &*self.ref_value() {
            Value::Number(_) | Value::Input(_) => vec![],
            Value::Add(a, b)
            | Value::Mul(a, b)
            | Value::Div(a, b)
            | Value::Mod(a, b)
            | Value::Eql(a, b, _) => vec![a.clone(), b.clone()],
        }
    }

    fn reduce(&mut self, factory: &Factory) -> Result<bool, &'static str> {
        let mut changed = false;
        let mut next_value: Option<Value> = None;
        let mut found = Equalities([None; 14]);

        loop {
            if next_value.is_some() {
                let mut self_ref_mut = self.ref_mut_value();
                let value = &mut *self_ref_mut;
                changed = true;
                let eq = {
                    let eq = self.rc_value.solved.borrow().clone();
                    eq.add(&found)
                };
                self.rc_value.solved.replace(eq);
                *value = next_value.take().unwrap();
            }

            match &*self.ref_value() {
                Value::Add(a, b) => {
                    let (a, b, reordered) = Self::reorder((a, b));

                    if reordered {
                        next_value = Some(Value::Add(a.clone(), b.clone()));
                        continue;
                    }

                    if *a.ref_value() == Value::Number(0) {
                        next_value = Some(b.clone_value());
                        continue;
                    }

                    if let (Value::Number(a), Value::Number(b)) = (&*a.ref_value(), &*b.ref_value())
                    {
                        next_value = Some(Value::Number(*a + *b));
                        continue;
                    }

                    if a == b {
                        next_value = Some(Value::Mul(
                            factory.try_new(
                                Value::Number(2),
                                Some(a.equalities().add(&b.equalities())),
                            )?,
                            a.clone(),
                        ));
                        continue;
                    }

                    if let (Value::Add(a1, a2), Value::Add(b1, b2)) =
                        (&*a.ref_value(), &*b.ref_value())
                    {
                        if a1 == a2 && a1 == b1 && a1 == b2 {
                            next_value = Some(Value::Mul(
                                factory.try_new(
                                    Value::Number(4),
                                    Some(
                                        a1.equalities()
                                            .add(&b1.equalities())
                                            .add(&a2.equalities())
                                            .add(&b2.equalities()),
                                    ),
                                )?,
                                a1.clone(),
                            ));
                            continue;
                        }
                        let ([a1, a2, b1, b2], _) = Self::reorder_4([a1, a2, b1, b2]);
                        next_value = Some(Value::Add(
                            a1.clone(),
                            factory.try_new(
                                Value::Add(
                                    a2.clone(),
                                    factory.try_new(
                                        Value::Add(b1.clone(), b2.clone()),
                                        Some(
                                            a1.equalities()
                                                .add(&b1.equalities())
                                                .add(&a2.equalities())
                                                .add(&b2.equalities()),
                                        ),
                                    )?,
                                ),
                                Some(a1.equalities()),
                            )?,
                        ));
                        continue;
                    }

                    if let (a, Value::Add(b1, b2)) = (a, &*b.ref_value()) {
                        if a == b1 && a == b2 {
                            next_value = Some(Value::Mul(
                                factory.try_new(
                                    Value::Number(3),
                                    Some(
                                        a.equalities().add(&b1.equalities()).add(&b2.equalities()),
                                    ),
                                )?,
                                a.clone(),
                            ));
                            continue;
                        }

                        let ([a, b1, b2], reordered) = Self::reorder_3([a, b1, b2]);
                        changed = changed || reordered;

                        let (reduced_ab1, is_reduced) = factory.new_is_reducible(
                            Value::Add(a.clone(), b1.clone()),
                            Some(a.equalities().add(&b1.equalities())),
                        )?;
                        if is_reduced {
                            next_value = Some(Value::Add(reduced_ab1, b2.clone()));
                            continue;
                        }

                        let (reduced_ab2, is_reduced) = factory.new_is_reducible(
                            Value::Add(a.clone(), b2.clone()),
                            Some(a.equalities().add(&b2.equalities())),
                        )?;
                        if is_reduced {
                            next_value = Some(Value::Add(reduced_ab2, b1.clone()));
                            continue;
                        }

                        if reordered {
                            next_value = Some(Value::Add(
                                a.clone(),
                                factory.try_new(
                                    Value::Add(b1.clone(), b2.clone()),
                                    Some(b1.equalities().add(&b2.equalities())),
                                )?,
                            ));
                            continue;
                        }
                    }

                    if let (Value::Mul(a1, a2), Value::Mul(b1, b2)) =
                        (&*a.ref_value(), &*b.ref_value())
                    {
                        if a1 == b1 {
                            next_value = Some(Value::Mul(
                                a1.clone(),
                                factory.try_new(
                                    Value::Add(a2.clone(), b2.clone()),
                                    Some(a2.equalities().add(&b2.equalities())),
                                )?,
                            ));
                            continue;
                        }
                        if a1 == b2 {
                            next_value = Some(Value::Mul(
                                a1.clone(),
                                factory.try_new(
                                    Value::Add(a2.clone(), b1.clone()),
                                    Some(a2.equalities().add(&b1.equalities())),
                                )?,
                            ));
                            continue;
                        }
                        if a2 == b1 {
                            next_value = Some(Value::Mul(
                                a2.clone(),
                                factory.try_new(
                                    Value::Add(a1.clone(), b2.clone()),
                                    Some(a1.equalities().add(&b2.equalities())),
                                )?,
                            ));
                            continue;
                        }
                        if a2 == b2 {
                            next_value = Some(Value::Mul(
                                a2.clone(),
                                factory.try_new(
                                    Value::Add(a1.clone(), b1.clone()),
                                    Some(a1.equalities().add(&b1.equalities())),
                                )?,
                            ));
                            continue;
                        }
                    }

                    if let Value::Mul(b1, b2) = &*b.ref_value() {
                        if a == b1 {
                            next_value = Some(Value::Mul(
                                a.clone(),
                                factory.try_new(
                                    Value::Add(
                                        factory.try_new(Value::Number(1), None)?,
                                        b2.clone(),
                                    ),
                                    Some(b2.equalities()),
                                )?,
                            ));
                            continue;
                        }
                        if a == b2 {
                            next_value = Some(Value::Mul(
                                a.clone(),
                                factory.try_new(
                                    Value::Add(
                                        factory.try_new(Value::Number(1), None)?,
                                        b1.clone(),
                                    ),
                                    Some(b1.equalities()),
                                )?,
                            ));
                            continue;
                        }
                    }
                }
                Value::Mul(a, b) => {
                    let (a, b, reordered) = Self::reorder((a, b));

                    if reordered {
                        next_value = Some(Value::Mul(a.clone(), b.clone()));
                        continue;
                    }

                    if *a.ref_value() == Value::Number(0) {
                        next_value = Some(Value::Number(0));
                        continue;
                    }

                    if let (Value::Number(a), Value::Number(b)) = (&*a.ref_value(), &*b.ref_value())
                    {
                        next_value = Some(Value::Number(a * b));
                        continue;
                    }

                    if *a.ref_value() == Value::Number(1) {
                        next_value = Some(b.ref_value().clone());
                        continue;
                    }

                    if let (Value::Mul(a1, a2), Value::Mul(b1, b2)) =
                        (&*a.ref_value(), &*b.ref_value())
                    {
                        let ([a1, a2, b1, b2], reordered) = Self::reorder_4([a1, a2, b1, b2]);

                        if reordered {
                            next_value = Some(Value::Mul(
                                a1.clone(),
                                factory.try_new(
                                    Value::Mul(
                                        a2.clone(),
                                        factory.try_new(
                                            Value::Mul(b1.clone(), b2.clone()),
                                            Some(b1.equalities().add(&b2.equalities())),
                                        )?,
                                    ),
                                    Some(a1.equalities()),
                                )?,
                            ));
                            continue;
                        }
                    }

                    if let (a, Value::Mul(b1, b2)) = (a, &*b.ref_value()) {
                        let ([a, b1, b2], reordered) = Self::reorder_3([a, b1, b2]);

                        if reordered {
                            next_value = Some(Value::Mul(
                                a.clone(),
                                factory.try_new(
                                    Value::Mul(b1.clone(), b2.clone()),
                                    Some(b1.equalities().add(&b2.equalities())),
                                )?,
                            ));
                            continue;
                        }

                        let (reduced_ab1, is_reduced) = factory.new_is_reducible(
                            Value::Mul(a.clone(), b1.clone()),
                            Some(a.equalities().add(&b1.equalities())),
                        )?;
                        if is_reduced {
                            next_value = Some(Value::Mul(reduced_ab1, b2.clone()));
                            continue;
                        }

                        let (reduced_ab2, is_reduced) = factory.new_is_reducible(
                            Value::Mul(a.clone(), b2.clone()),
                            Some(a.equalities().add(&b2.equalities())),
                        )?;
                        if is_reduced {
                            next_value = Some(Value::Mul(reduced_ab2, b1.clone()));
                            continue;
                        }
                    }

                    if let (a, Value::Div(b1, b2)) = (a, &*b.ref_value()) {
                        let (reduced_ab1, is_reduced) = factory.new_is_reducible(
                            Value::Mul(a.clone(), b1.clone()),
                            Some(a.equalities().add(&b1.equalities())),
                        )?;
                        if is_reduced {
                            println!(
                                "reduced:\n- {}\n- {} =\n - {}\n - {}\n\n=>\n\n- {}\n- {}\n",
                                a, b, b1, b2, reduced_ab1, b2
                            );
                            next_value = Some(Value::Div(reduced_ab1, b2.clone()));
                            continue;
                        }

                        let (reduced_ab2, is_reduced) = factory.new_is_reducible(
                            Value::Div(a.clone(), b2.clone()),
                            Some(a.equalities().add(&b2.equalities())),
                        )?;
                        if is_reduced {
                            next_value = Some(Value::Mul(reduced_ab2, b1.clone()));
                            continue;
                        }
                    }
                }
                Value::Input(_) | Value::Number(_) => {}
                Value::Div(a, b) => {
                    if *b.ref_value() == Value::Number(0) {
                        return Err("division by zero");
                    }
                    if *a.ref_value() == Value::Number(0) {
                        next_value = Some(Value::Number(0));
                        continue;
                    }

                    if *b.ref_value() == Value::Number(1) {
                        next_value = Some(a.ref_value().clone());
                        continue;
                    }

                    if let (Value::Number(a), Value::Number(b)) = (&*a.ref_value(), &*b.ref_value())
                    {
                        next_value =
                            Some(Value::Number(a.checked_div(*b).ok_or("Division by zero")?));
                        continue;
                    }

                    if let Value::Mul(a1, a2) = &*a.ref_value() {
                        let (reduced_a1b, is_reduced) = factory.new_is_reducible(
                            Value::Div(a1.clone(), b.clone()),
                            Some(a1.equalities().add(&b.equalities())),
                        )?;
                        if is_reduced {
                            next_value = Some(Value::Mul(reduced_a1b, a2.clone()));
                            continue;
                        }

                        let (reduced_a2b, is_reduced) = factory.new_is_reducible(
                            Value::Div(a2.clone(), b.clone()),
                            Some(a2.equalities().add(&b.equalities())),
                        )?;
                        if is_reduced {
                            next_value = Some(Value::Mul(reduced_a2b, a1.clone()));
                            continue;
                        }
                    }
                }
                Value::Mod(a, b) => {
                    if let Value::Number(b) = &*b.ref_value() {
                        if *b <= 0 {
                            return Err("modulo by b <= 0");
                        }
                        if let Value::Number(a) = &*a.ref_value() {
                            next_value = Some(Value::Number(*a % *b));
                            continue;
                        }
                    }

                    if *a.ref_value() == Value::Number(0) {
                        next_value = Some(Value::Number(0));
                        continue;
                    }
                }
                Value::Eql(a, b, found_) => {
                    let (a, b, reordered) = Self::reorder((a, b));

                    if reordered {
                        next_value = Some(Value::Eql(a.clone(), b.clone(), *found_));
                        continue;
                    }

                    if a == b {
                        if let Some(idx) = found_ {
                            found.win(*idx)
                        }
                        next_value = Some(Value::Number(1));
                        continue;
                    }

                    if let (Value::Number(a), Value::Number(b)) = (&*a.ref_value(), &*b.ref_value())
                    {
                        if a != b {
                            if let Some(idx) = found_ {
                                found.fail(*idx)
                            }
                            next_value = Some(Value::Number(0));
                            continue;
                        }
                    }

                    if let (Value::Number(a), Value::Input(_)) = (&*a.ref_value(), &*b.ref_value())
                    {
                        if !(1..=9).contains(a) {
                            if let Some(idx) = found_ {
                                found.fail(*idx)
                            }
                            next_value = Some(Value::Number(0));
                            continue;
                        }
                    }

                    let (a, b, reordered) = Self::reorder((a, b));
                    if reordered {
                        next_value = Some(Value::Eql(a.clone(), b.clone(), *found_));
                        continue;
                    }
                }
            };

            return Ok(changed);
        }
    }
}

impl FromStr for VariableOrNumber {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(variable) = s.parse() {
            Ok(Self::Variable(variable))
        } else {
            Ok(Self::Number(s.parse().map_err(|_| ())?))
        }
    }
}

#[derive(Debug)]
enum Instruction {
    Inp(Variable),
    Add(Variable, VariableOrNumber),
    Mul(Variable, VariableOrNumber),
    Div(Variable, VariableOrNumber),
    Mod(Variable, VariableOrNumber),
    Eql(Variable, VariableOrNumber, bool),
}

impl FromStr for Instruction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split(' ').collect_vec();
        let result = match parts[0] {
            "inp" => Self::Inp(parts[1].parse()?),
            "add" => Self::Add(parts[1].parse()?, parts[2].parse()?),
            "mul" => Self::Mul(parts[1].parse()?, parts[2].parse()?),
            "div" => Self::Div(parts[1].parse()?, parts[2].parse()?),
            "mod" => Self::Mod(parts[1].parse()?, parts[2].parse()?),
            "eql" => {
                let part1 = parts[1].parse()?;
                let part2 = parts[2].parse()?;
                let is_var = match part2 {
                    VariableOrNumber::Variable(_) => true,
                    VariableOrNumber::Number(_) => false,
                };
                Self::Eql(part1, part2, is_var)
            }
            _ => return Err(()),
        };
        Ok(result)
    }
}

impl Instruction {
    fn solve_placeholder(
        placeholder: &VariableOrNumber,
        variables: &mut HashMap<Variable, ValueRef>,
        factory: &Factory,
    ) -> Result<ValueRef, &'static str> {
        match placeholder {
            VariableOrNumber::Variable(name) => {
                Ok(variables.get(name).ok_or("unknown variable")?.clone())
            }
            &VariableOrNumber::Number(n) => Ok(factory.try_new(Value::Number(n), None))?,
        }
    }

    fn execute(
        &self,
        input_count: &mut u8,
        eql_count: &mut u8,
        variables: &mut HashMap<Variable, ValueRef>,
        inputs: &[Option<i64>; 14],
        factory: &Factory,
    ) -> Result<(), &'static str> {
        match self {
            &Instruction::Inp(variable) => {
                if let Some(value) = inputs[*input_count as usize] {
                    variables.insert(variable, factory.try_new(Value::Number(value), None)?);
                } else {
                    let mut result = factory.try_new(Value::Input(*input_count), None)?;
                    result.reduce(factory)?;
                    variables.insert(variable, result);
                }

                *input_count += 1;
            }
            Instruction::Add(variable, placeholder) => {
                let value = Self::solve_placeholder(placeholder, variables, factory)?;

                let mut result = factory.try_new(
                    Value::Add(variables.get(variable).ok_or("unknown variable")?.clone(), value),
                    None,
                )?;
                result.reduce(factory)?;
                variables.insert(*variable, result);
            }
            Instruction::Mul(variable, placeholder) => {
                let value = Self::solve_placeholder(placeholder, variables, factory)?;
                let mut result = factory.try_new(
                    Value::Mul(variables.get(variable).ok_or("unknown variable")?.clone(), value),
                    None,
                )?;
                result.reduce(factory)?;
                variables.insert(*variable, result);
            }
            Self::Div(variable, placeholder) => {
                let value = Self::solve_placeholder(placeholder, variables, factory)?;
                let mut result = factory.try_new(
                    Value::Div(variables.get(variable).ok_or("unknown variable")?.clone(), value),
                    None,
                )?;
                result.reduce(factory)?;
                variables.insert(*variable, result);
            }
            Self::Mod(variable, placeholder) => {
                let value = Self::solve_placeholder(placeholder, variables, factory)?;
                let mut result = factory.try_new(
                    Value::Mod(variables.get(variable).ok_or("unknown variable")?.clone(), value),
                    None,
                )?;
                result.reduce(factory)?;
                variables.insert(*variable, result);
            }
            Self::Eql(variable, placeholder, is_var) => {
                let value = Self::solve_placeholder(placeholder, variables, factory)?;
                let var = if *is_var { Some(*eql_count) } else { None };
                let mut result = factory.try_new(
                    Value::Eql(
                        variables.get(variable).ok_or("unknown variable")?.clone(),
                        value,
                        var,
                    ),
                    None,
                )?;
                result.reduce(factory)?;
                variables.insert(*variable, result);
                if *is_var {
                    *eql_count += 1;
                }
            }
        }
        Ok(())
    }
}

fn inputs_to_string(inputs: &[Option<i64>; 14], replace_1: bool) -> String {
    inputs
        .iter()
        .map(|x| match x {
            Some(x) => x.to_string(),
            None => replace_1.then(|| "1").or(Some(" ")).unwrap().to_string(),
        })
        .collect_vec()
        .join("")
}

#[derive(Clone, Debug)]
struct Equalities([Option<bool>; 14]);

impl PartialOrd for Equalities {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let self_score: (usize, usize) =
            self.0.iter().flatten().fold((0, 0), |(wins, fails), v| {
                if *v {
                    (wins + 1, fails)
                } else {
                    (wins, fails + 1)
                }
            });
        let other_score: (usize, usize) =
            other.0.iter().flatten().fold((0, 0), |(wins, fails), v| {
                if *v {
                    (wins + 1, fails)
                } else {
                    (wins, fails + 1)
                }
            });
        self_score.partial_cmp(&other_score)
    }
}

impl Eq for Equalities {}

impl Ord for Equalities {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialEq for Equalities {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl Display for Equalities {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{:?}", self.0))
    }
}

impl Equalities {
    fn add(self, other: &Self) -> Self {
        let mut result = self;
        for (idx, v) in other.0.iter().enumerate() {
            match v {
                None => {}
                Some(true) => {
                    if result.0[idx] == Some(false) {
                        unreachable!()
                    }
                    result.0[idx] = Some(true);
                }
                Some(false) => {
                    if result.0[idx] == Some(true) {
                        unreachable!()
                    }
                    result.0[idx] = Some(false);
                }
            }
        }
        result
    }

    fn win(&mut self, idx: u8) {
        if self.0[idx as usize] == Some(false) {
            unreachable!()
        }
        self.0[idx as usize] = Some(true)
    }

    fn fail(&mut self, idx: u8) {
        if self.0[idx as usize] == Some(true) {
            unreachable!()
        }
        self.0[idx as usize] = Some(false)
    }
}

#[derive(Clone)]
struct Step {
    inputs: [Option<i64>; 14],
    value: Option<i64>,
    dependencies: u8,
    found: Equalities,
}

impl Hash for Step {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for x in self.normalized_input().iter() {
            x.hash(state)
        }
    }
}

impl PartialEq<Self> for Step {
    fn eq(&self, other: &Self) -> bool {
        self.normalized_input().eq(&other.normalized_input())
    }
}

impl Eq for Step {}

impl Ord for Step {
    fn cmp(&self, other: &Self) -> Ordering {
        let value_order = || self.value.cmp(&other.value).reverse();
        let input_order = || self.inputs.cmp(&other.inputs).reverse();
        let dependencies = || self.dependencies.cmp(&other.dependencies);
        let found = || self.found.cmp(&other.found);

        found()
            .then_with(input_order)
            .then_with(dependencies)
            .then_with(value_order)
    }
}

impl PartialOrd for Step {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Step {
    fn normalized_input(&self) -> [i64; 14] {
        let mut inputs = [1; 14];
        for (i, v) in self.inputs.iter().enumerate() {
            match v {
                None => inputs[i] = 1,
                Some(v) => inputs[i] = *v,
            }
        }
        inputs
    }

    fn new(variable: &ValueRef, inputs: [Option<i64>; 14], factory: &Factory) -> Self {
        let value = Self::value(&inputs, variable, factory).ok();
        let result = variable.value_for_inputs(&inputs, factory).unwrap();
        let dependencies = factory.set_input_dependencies(&result).iter().filter(|x| **x).count() as u8;
        Self { inputs, value, dependencies, found: result.equalities() }
    }

    fn next(self, variable: &ValueRef, factory: &Factory) -> Vec<(u8, Self)> {
        let input_dependencies = {
            let result = variable.value_for_inputs(&self.inputs, factory).unwrap();
            factory.set_input_dependencies(&result)
        };

        seed(&self.inputs)
            .filter(|(i, _)| input_dependencies[*i as usize])
            .map(|(idx, v)| {
                let mut inputs = self.inputs;
                inputs[idx as usize] = v;
                let step = Self::new(variable, inputs, factory);
                (idx as u8, step)
            })
            .collect()
    }

    fn value(
        inputs: &[Option<i64>; 14],
        variable: &ValueRef,
        factory: &Factory,
    ) -> Result<i64, &'static str> {
        let mut inputs = *inputs;
        for i in inputs.iter_mut().filter(|v| v.is_none()) {
            *i = Some(1)
        }
        let value = variable.value_for_inputs(&inputs, factory)?;
        let result = match *value.ref_value() {
            Value::Number(n) => Ok(n),
            _ => unreachable!(),
        };
        result
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum Deduction {
    Unknown,
    Impossible,
    In { min: i64, max: i64 },
}


fn seed(inputs: &[Option<i64>; 14]) -> impl Iterator<Item = (u8, Option<i64>)> + '_ {
    (0..14)
        .filter_map(|idx| {
            if inputs[idx].is_some() {
                None
            } else {
                Some((1..=9).map(move |i| (idx as u8, Some(i))))
            }
        })
        .take(1)
        .flatten()
}

fn main() -> Result<(), &'static str> {
    let stdin = io::stdin();
    let program: Vec<Instruction> = stdin
        .lock()
        .lines()
        .flatten()
        .map(|line| line.parse())
        .collect::<Result<_, _>>()
        .map_err(|_| "cannot parse input")?;

    let factory = Factory {
        cache: RefCell::new(LruCache::new(1000)),
        input_dependencies: RefCell::new(LruCache::new(1000)),
        inputs_resolves: RefCell::new(LruCache::new(1000)),
        deductions: RefCell::new(LruCache::new(1000)),
    };

    let mut input_count = 0u8;
    let mut eql_count = 0u8;
    let mut variables = HashMap::from([
        (Variable::X, factory.try_new(Value::Number(0), None)?),
        (Variable::Y, factory.try_new(Value::Number(0), None)?),
        (Variable::Z, factory.try_new(Value::Number(0), None)?),
        (Variable::W, factory.try_new(Value::Number(0), None)?),
    ]);

    for (line, instruction) in program.iter().enumerate() {
        if let Err(s) = instruction.execute(
            &mut input_count,
            &mut eql_count,
            &mut variables,
            &Default::default(),
            &factory,
        ) {
            println!("{}: {:?} at line {}", s, instruction, line);
            break;
        }
    }

    let mut steps = BinaryHeap::new();

    for (idx, v) in seed(&[None; 14]) {
        let mut inputs = [None; 14];
        inputs[idx as usize] = v;
        steps.push(Step::new(&variables[&Variable::Z], inputs, &factory))
    }

    let mut smallest = [Some(10); 14];

    let mut record_str = "XXXXXXXXXXXXXX".to_string();
    while let Some(step) = steps.pop() {
        if let Some(value) = &step.value {
            if *value == 0 {
                if step.inputs < smallest {
                    smallest = step.inputs;
                    record_str = inputs_to_string(&step.inputs, true);
                }
                continue;
            }
        }

        steps.extend(step.next(&variables[&Variable::Z], &factory).into_iter().filter_map(
            |(_, v)| {
                let this_input = inputs_to_string(&v.inputs, true);
                if this_input >= record_str {
                    return None;
                }

                if let Ok(result) = variables[&Variable::Z].solve(&factory, &v.inputs) {
                    match factory.set_deduction(&result) {
                        Deduction::Unknown => {}
                        Deduction::Impossible => {}
                        Deduction::In { min, max} => {
                            if !(min..=max).contains(&0) {
                                return None;
                            }
                        }
                    }
                }
                Some(v)
            },
        ))
    }

    println!("{}", record_str);
    Ok(())
}
