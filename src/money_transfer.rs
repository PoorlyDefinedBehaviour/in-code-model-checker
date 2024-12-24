use std::collections::HashMap;

#[derive(Debug, Clone)]
struct State {
    balances: HashMap<String, i64>,
    pc: HashMap<i64, String>,
}

impl std::hash::Hash for State {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let mut keys: Vec<_> = self.balances.keys().collect();
        keys.sort_unstable();
        for key in keys {
            key.hash(state);
            self.balances.get(key).unwrap().hash(state);
        }

        let mut keys: Vec<_> = self.pc.keys().collect();
        keys.sort_unstable();
        for key in keys {
            key.hash(state);
            self.pc.get(key).unwrap().hash(state);
        }
    }
}

fn deposit(s: &mut State, account: &str, amount: i64) {
    let balance = s.balances.get_mut(account).unwrap();
    *balance += amount;
}

fn start_transfer(s: &mut State, r: i64, from: &str, to: &str, amount: i64) {
    if s.pc.get(&r).unwrap() != "start_transfer" {
        return;
    }

    let balance = s.balances.get_mut(from).unwrap();
    if *balance - amount >= 0 {
        s.pc.insert(r, "complete_transfer".to_owned());
    }
}

fn complete_transfer(s: &mut State, r: i64, from: &str, to: &str, amount: i64) {
    if s.pc.get(&r).unwrap() != "complete_transfer" {
        return;
    }

    *s.balances.get_mut(from).unwrap() -= amount;
    *s.balances.get_mut(to).unwrap() += amount;

    s.pc.insert(r, "done".to_owned());
}

fn consistency(s: &State) -> bool {
    s.balances.iter().all(|(_, balance)| *balance >= 0)
}

#[cfg(test)]
mod tests {
    use crate::model_checker::ModelChecker;

    use super::*;

    #[test]
    fn money_transfer() {
        let mut checker = ModelChecker::new();

        checker
            .action("deposit", |c: &mut ModelChecker<State>, s: &mut State| {
                c.there_exists(s.balances.iter(), |s, (account, _balance)| {
                    deposit(s, account, 1)
                })
            })
            .action("start_transfer", |c, s| {
                c.there_exists(1..=2, |s, r| start_transfer(s, r, "Alice", "Bob", 1));
            })
            .action("complete_transfer", |c, s: &mut State| {
                c.there_exists(1..=2, |s, r| complete_transfer(s, r, "Alice", "Bob", 1));
            })
            .invariant("consistency", consistency);

        checker.run(State {
            balances: HashMap::from([("Alice".to_string(), 1), ("Bob".to_string(), 1)]),
            pc: HashMap::from([
                (1, "start_transfer".to_owned()),
                (2, "start_transfer".to_owned()),
            ]),
        });
    }
}
