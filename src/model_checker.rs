use std::{
    cell::RefCell,
    collections::{hash_map::DefaultHasher, HashMap, HashSet, VecDeque},
    hash::Hasher,
    rc::Rc,
};

pub struct Action<State> {
    pub name: String,
    pub f: Rc<dyn Fn(&mut ModelChecker<State>, &mut State)>,
}

pub struct Invariant<State> {
    pub name: String,
    pub f: Box<dyn Fn(&State) -> bool>,
}

#[derive(Debug, Clone)]
pub struct Pending<State> {
    pub generated_by_action: String,
    pub previous: Option<Box<Pending<State>>>,
    pub current: State,
}

pub struct ModelChecker<State> {
    pub queue: RefCell<VecDeque<Pending<State>>>,
    pub current: Option<Pending<State>>,
    pub current_action: usize,
    pub seen: RefCell<HashSet<u64>>,
    pub actions: Vec<Action<State>>,
    pub invariants: Vec<Invariant<State>>,
}

fn hash(v: impl std::hash::Hash) -> u64 {
    let mut hasher = DefaultHasher::new();

    v.hash(&mut hasher);

    hasher.finish()
}

impl<State: std::hash::Hash + std::fmt::Debug + Clone> ModelChecker<State> {
    pub fn new() -> Self {
        Self {
            queue: RefCell::new(VecDeque::new()),
            current: None,
            current_action: 0,
            seen: RefCell::new(HashSet::new()),
            actions: Vec::new(),
            invariants: Vec::new(),
        }
    }

    pub fn action(
        &mut self,
        name: impl Into<String>,
        action: impl Fn(&mut ModelChecker<State>, &mut State) + 'static,
    ) -> &mut Self {
        self.actions.push(Action {
            name: name.into(),
            f: Rc::new(action),
        });
        self
    }

    pub fn invariant(
        &mut self,
        name: impl Into<String>,
        invariant: impl Fn(&State) -> bool + 'static,
    ) -> &mut Self {
        self.invariants.push(Invariant {
            name: name.into(),
            f: Box::new(invariant),
        });
        self
    }

    pub fn run(&mut self, initial_state: State) {
        self.seen.borrow_mut().insert(hash(&initial_state));
        self.queue.borrow_mut().push_back(Pending {
            generated_by_action: "Init".to_string(),
            previous: None,
            current: initial_state,
        });

        while !self.queue.borrow().is_empty() {
            self.step();
        }
    }

    fn step(&mut self) {
        let pending = self.queue.borrow_mut().pop_front().unwrap();

        for invariant in self.invariants.iter() {
            if !(invariant.f)(&pending.current) {
                let behavior = build_behavior(&pending);
                println!("invariant {} violated by {behavior:#?}", &invariant.name);
                std::process::exit(1);
            }
        }

        self.current = Some(pending);

        for i in 0..self.actions.len() {
            self.current_action = i;
            let mut new_state = {
                let current = self.current.as_mut().unwrap();
                current.current.clone()
            };

            let f = Rc::clone(&self.actions[i].f);
            (f)(self, &mut new_state);

            let hash = hash(&new_state);
            if !self.seen.borrow().contains(&hash) {
                self.seen.borrow_mut().insert(hash);

                self.queue.borrow_mut().push_back(Pending {
                    generated_by_action: self.actions[i].name.clone(),
                    previous: self.current.clone().map(Box::new),
                    current: new_state,
                });
            }
        }
    }

    pub fn there_exists<V>(&self, xs: impl Iterator<Item = V>, f: impl Fn(&mut State, V)) {
        for x in xs {
            let mut new_state = self.current.as_ref().unwrap().current.clone();

            f(&mut new_state, x);

            let hash = hash(&new_state);

            if !self.seen.borrow().contains(&hash) {
                self.seen.borrow_mut().insert(hash);

                self.queue.borrow_mut().push_back(Pending {
                    generated_by_action: self.actions[self.current_action].name.clone(),
                    previous: self.current.clone().map(Box::new),
                    current: new_state,
                });
            }
        }
    }
}

trait ThereExists {
    type Value;
}

impl<K, V> ThereExists for HashMap<K, V> {
    type Value = (K, V);
}

fn build_behavior<State: Clone>(s: &Pending<State>) -> Vec<(String, State)> {
    let mut steps = vec![(s.generated_by_action.clone(), s.current.clone())];

    let mut previous = s.previous.clone();
    while let Some(p) = previous.take() {
        steps.push((p.generated_by_action, p.current));
        previous = p.previous;
    }

    steps.reverse();

    steps
}
