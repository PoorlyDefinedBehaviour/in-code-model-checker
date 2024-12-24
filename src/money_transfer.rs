use std::collections::HashMap;

#[derive(Debug, Clone)]
struct State {
    balances: HashMap<String, i64>,
}

impl std::hash::Hash for State {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let mut keys: Vec<_> = self.balances.keys().collect();
        keys.sort_unstable();
        for key in keys {
            key.hash(state);
            self.balances.get(key).unwrap().hash(state);
        }
    }
}

fn deposit(s: &mut State, account: &str, amount: i64) {
    let balance = s.balances.get_mut(account).unwrap();
    *balance += amount;
}

fn transfer(s: &mut State, from: &str, to: &str, amount: i64) {
    let balance = s.balances.get_mut(from).unwrap();
    *balance -= amount;

    let balance = s.balances.get_mut(to).unwrap();
    *balance += amount;
}

fn consistency(s: &State) -> bool {
    if *s.balances.get("Alice").unwrap() == 1 && *s.balances.get("Bob").unwrap() == 3 {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

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
            .action("transfer", |c, s| {
                let accounts: HashSet<_> = s.balances.keys().collect();
                c.there_exists(accounts.iter(), |_, from| {
                    c.there_exists(accounts.iter().filter(|acc| acc != &from), |s, to| {
                        transfer(s, from, to, 1)
                    })
                });
            })
            .invariant("consistency", consistency);

        checker.run(State {
            balances: HashMap::from([("Alice".to_string(), 2), ("Bob".to_string(), 1)]),
        });
    }
}
