use crate::model_checker::ModelChecker;

#[derive(Debug, Clone, Hash)]
struct State {
    small: u8,
    big: u8,
}

fn fill_small(c: &mut ModelChecker<State>, s: &mut State) {
    s.small = 3;
}

fn fill_big(c: &mut ModelChecker<State>, s: &mut State) {
    s.big = 5;
}

fn empty_small(c: &mut ModelChecker<State>, s: &mut State) {
    s.small = 0;
}

fn empty_big(c: &mut ModelChecker<State>, s: &mut State) {
    s.big = 0;
}

fn move_from_big_to_small(c: &mut ModelChecker<State>, s: &mut State) {
    let amount = std::cmp::min(3 - s.small, s.big);
    s.small += amount;
    s.big -= amount;
}

fn move_from_small_to_big(c: &mut ModelChecker<State>, s: &mut State) {
    let amount = std::cmp::min(5 - s.big, s.small);
    s.small -= amount;
    s.big += amount;
}

fn gallon_never_has_4_liters(s: &State) -> bool {
    s.big != 4
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::model_checker::ModelChecker;

    #[test]
    fn die_hard() {
        let mut checker = ModelChecker::new();

        checker
            .action("fill_small", fill_small)
            .action("fill_big", fill_big)
            .action("empty_small", empty_small)
            .action("empty_big", empty_big)
            .action("move_from_big_to_small", move_from_big_to_small)
            .action("move_from_small_to_big", move_from_small_to_big)
            .invariant("gallon_never_has_4_liters", gallon_never_has_4_liters);

        checker.run(State { small: 0, big: 0 });
    }
}
