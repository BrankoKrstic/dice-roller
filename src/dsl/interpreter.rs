use rand::{rngs::ThreadRng, RngExt};
use thiserror::Error;

use crate::dsl::parser::{
    Ast, BinaryOp, Condition, Dice, DiceKind, DiceModifier, ModifierOp, SortOrder, UnaryOp,
};

pub trait DiceRng {
    fn roll_inclusive(&mut self, start: i64, end: i64) -> i64;
}

pub struct CryptoDiceRng {
    rng: ThreadRng,
}

impl DiceRng for CryptoDiceRng {
    fn roll_inclusive(&mut self, start: i64, end: i64) -> i64 {
        self.rng.random_range(start..=end)
    }
}

pub struct Interpreter<R> {
    rng: R,
}

#[derive(Debug)]
pub struct DiceRoll {
    dropped: bool,
    result: i64,
}

impl std::fmt::Display for DiceRoll {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.result, if self.dropped { "d" } else { "" })
    }
}

#[derive(Debug)]
pub enum EvalResult {
    Number {
        value: i64,
    },
    DiceRollGroup {
        counted: Option<i64>,
        value: i64,
        rolls: Vec<DiceRoll>,
    },
    Unary {
        op: UnaryOp,
        value: i64,
        child: Box<EvalResult>,
    },
    Binary {
        op: BinaryOp,
        value: i64,
        lhs: Box<EvalResult>,
        rhs: Box<EvalResult>,
    },
}

impl EvalResult {
    pub fn total(&self) -> i64 {
        match self {
            EvalResult::Number { value } => *value,
            EvalResult::DiceRollGroup { counted, value, .. } => *counted.as_ref().unwrap_or(value),
            EvalResult::Unary { value, .. } => *value,
            EvalResult::Binary { value, .. } => *value,
        }
    }
}

impl std::fmt::Display for EvalResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvalResult::Number { value } => write!(f, "{}", *value),
            EvalResult::DiceRollGroup {
                counted,
                value,
                rolls,
            } => {
                let val = *counted.as_ref().unwrap_or(value);
                if rolls.len() <= 1 {
                    return write!(f, "{}", val);
                }

                let mut out = format!("({}", rolls[0]);

                for roll in rolls.iter().skip(1) {
                    out.push_str(&format!(", {}", roll));
                }

                out.push_str(&format!(") {}", val));

                write!(f, "{}", out)
            }
            EvalResult::Unary { op, value, child } => {
                write!(
                    f,
                    "{}({child}) -> {value}",
                    match op {
                        UnaryOp::Negate => '-',
                    }
                )
            }
            EvalResult::Binary {
                op,
                value,
                lhs,
                rhs,
            } => {
                let op = match op {
                    BinaryOp::Add => "+",
                    BinaryOp::Subtract => "-",
                    BinaryOp::Multiply => "*",
                    BinaryOp::Divide => "/",
                };

                write!(f, "({lhs} {op} {rhs}) -> {value}",)
            }
        }
    }
}

impl EvalResult {
    fn get_val(&self) -> i64 {
        match self {
            EvalResult::Number { value } => *value,
            EvalResult::DiceRollGroup { value, counted, .. } => *counted.as_ref().unwrap_or(value),
            EvalResult::Unary { value, .. } => *value,
            EvalResult::Binary { value, .. } => *value,
        }
    }
}

#[derive(Debug, Error)]
pub enum InterpreterError {
    #[error("Division by zero attempted")]
    DivideByZero,
}

impl<R: DiceRng> Interpreter<R> {
    pub fn new(rng: R) -> Self {
        Self { rng }
    }
    pub fn roll_dice(&mut self, dice: DiceKind) -> i64 {
        match dice {
            DiceKind::DFudge => self.rng.roll_inclusive(-1, 1),
            x => self.rng.roll_inclusive(1, x.max_val() as i64),
        }
    }

    fn eval_dice_group(&mut self, dice: &Dice) -> Result<EvalResult, InterpreterError> {
        let mut out = Vec::with_capacity(dice.count as usize);

        for _ in 0..dice.count {
            out.push(DiceRoll {
                dropped: false,
                result: self.roll_dice(dice.kind),
            });
        }

        for m in &dice.modifiers {
            match m {
                DiceModifier::Unique => {
                    let mut reroll_count = 0;
                    for i in 0..out.len() {
                        if out[0..i].iter().any(|r| r.result == out[i].result) {
                            out[i].dropped = true;
                            reroll_count += 1;
                        }
                    }
                    while reroll_count > 0 {
                        let roll = self.roll_dice(dice.kind);
                        let duplicate = out.iter().any(|r| r.result == roll);

                        out.push(DiceRoll {
                            dropped: duplicate,
                            result: roll,
                        });
                        if !duplicate {
                            reroll_count -= 1;
                        }
                    }
                }
                DiceModifier::Explode { condition, count } => {
                    let mut idx = 0;
                    let target = condition.target as i64;

                    let mut match_cond = |x| match condition.op {
                        ModifierOp::Greater => x > target,
                        ModifierOp::Less => x < target,
                        ModifierOp::GreaterEqual => x >= target,
                        ModifierOp::LessEqual => x <= target,
                        ModifierOp::Equal => x == target,
                        _ => {
                            idx += 1;
                            idx < target
                        }
                    };

                    let mut max_count = count.unwrap_or(1) as usize;

                    let mut count = out.iter().filter(|r| match_cond(r.result)).count();

                    while count > 0 && max_count > 0 {
                        let roll = self.roll_dice(dice.kind);
                        if !match_cond(roll) {
                            count -= 1;
                        } else {
                            max_count -= 1;
                        }
                        out.push(DiceRoll {
                            dropped: false,
                            result: roll,
                        })
                    }
                }
                DiceModifier::Reroll { times, condition } => {
                    for _ in 0..*times {
                        let indices = find_cond_indices(&out[..], *condition);
                        if indices.is_empty() {
                            break;
                        }
                        for i in indices {
                            out[i].dropped = true;
                            let reroll = self.roll_dice(dice.kind);
                            out.push(DiceRoll {
                                dropped: false,
                                result: reroll,
                            });
                        }
                    }
                }
                _ => {}
            }
        }

        for m in &dice.modifiers {
            match m {
                DiceModifier::Keep { condition } => {
                    let indices = find_cond_indices(&out[..], *condition);
                    for (i, el) in out.iter_mut().enumerate() {
                        el.dropped = !indices.contains(&i);
                    }
                }
                DiceModifier::Drop { condition } => {
                    let indices = find_cond_indices(&out[..], *condition);
                    for (i, el) in out.iter_mut().enumerate() {
                        el.dropped = indices.contains(&i);
                    }
                }

                _ => {}
            }
        }

        for m in &dice.modifiers {
            match m {
                DiceModifier::Sort(sort_order) => {
                    out.sort_unstable_by_key(|x| {
                        if matches!(sort_order, SortOrder::Dsc) {
                            -x.result
                        } else {
                            x.result
                        }
                    });
                }

                DiceModifier::Min(min) => {
                    for roll in &mut out {
                        roll.result = roll.result.max(*min as i64);
                    }
                }
                DiceModifier::Max(max) => {
                    for roll in &mut out {
                        roll.result = roll.result.min(*max as i64);
                    }
                }
                _ => {}
            }
        }
        let mut counted = None;
        for m in &dice.modifiers {
            if let &DiceModifier::Count { condition } = m {
                if let Some(condition) = condition {
                    counted = Some(find_cond_indices(&out[..], condition).len() as i64);
                } else {
                    counted = Some(out.iter().filter(|x| !x.dropped).count() as i64);
                }
            }
        }
        Ok(EvalResult::DiceRollGroup {
            counted,
            value: out.iter().filter(|r| !r.dropped).map(|r| r.result).sum(),
            rolls: out,
        })
    }
    pub fn eval_ast(&mut self, ast: &Ast) -> Result<EvalResult, InterpreterError> {
        let res = match ast {
            Ast::Number(num) => EvalResult::Number { value: *num as i64 },
            Ast::Dice(dice) => self.eval_dice_group(dice)?,
            Ast::Unary { op, ast } => {
                let inner = self.eval_ast(ast)?;
                let inner_val = inner.get_val();

                let value = match op {
                    UnaryOp::Negate => -inner_val,
                };
                EvalResult::Unary {
                    op: *op,
                    value,
                    child: inner.into(),
                }
            }
            Ast::Binary { op, lhs, rhs } => {
                let lhs = self.eval_ast(lhs)?;
                let rhs = self.eval_ast(rhs)?;
                let lhs_val = lhs.get_val();
                let rhs_val = rhs.get_val();

                let value = match op {
                    BinaryOp::Add => lhs_val + rhs_val,
                    BinaryOp::Subtract => lhs_val - rhs_val,
                    BinaryOp::Multiply => lhs_val * rhs_val,
                    BinaryOp::Divide => lhs_val
                        .checked_div_euclid(rhs_val)
                        .ok_or(InterpreterError::DivideByZero)?,
                };
                EvalResult::Binary {
                    op: *op,
                    value,
                    lhs: lhs.into(),
                    rhs: rhs.into(),
                }
            }
        };
        Ok(res)
    }
}

fn find_cond_indices(nums: &[DiceRoll], cond: Condition) -> Vec<usize> {
    let mut out = nums
        .iter()
        .enumerate()
        .filter(|n| !n.1.dropped)
        .collect::<Vec<_>>();
    let target = cond.target as i64;
    match cond.op {
        ModifierOp::Lowest => {
            out.sort_unstable_by_key(|x| x.1.result);
            out.into_iter().take(target as usize).map(|x| x.0).collect()
        }
        ModifierOp::Highest => {
            out.sort_unstable_by_key(|x| -x.1.result);
            out.into_iter().take(target as usize).map(|x| x.0).collect()
        }
        x => out
            .into_iter()
            .filter(|val| match x {
                ModifierOp::Greater => val.1.result > target,
                ModifierOp::Less => val.1.result < target,
                ModifierOp::GreaterEqual => val.1.result >= target,
                ModifierOp::LessEqual => val.1.result <= target,
                ModifierOp::Equal => val.1.result == target,
                _ => unreachable!(),
            })
            .map(|x| x.0)
            .collect(),
    }
}

#[cfg(test)]
mod tests {

    use crate::dsl::{
        interpreter::{EvalResult, Interpreter, InterpreterError},
        parser::Parser,
    };

    use super::DiceRng;

    #[derive(Debug)]
    struct StubRng {
        values: Vec<i64>,
        index: usize,
    }

    impl StubRng {
        fn new(values: Vec<i64>) -> Self {
            Self { values, index: 0 }
        }
    }

    pub fn stub_roll(expr: &str, rng: impl DiceRng) -> Result<EvalResult, InterpreterError> {
        let mut p = Parser::new(expr);

        let mut intr = Interpreter::new(rng);

        let parsed = p.parse().expect("parse should succeed");

        intr.eval_ast(&parsed)
    }

    impl DiceRng for StubRng {
        fn roll_inclusive(&mut self, start: i64, end: i64) -> i64 {
            let value = self.values.get(self.index).copied().unwrap();
            self.index += 1;

            if value < start || value > end {
                panic!("stub value {value} out of range {start}..={end}");
            }

            value
        }
    }

    #[test]
    fn evaluates_basic_expression() {
        let rng = StubRng::new(vec![3, 6, 2]);
        let result = stub_roll("2d10 + 1d6 + 5", rng).expect("roll should succeed");

        assert_eq!(result.total(), 16);
        assert!(result.to_string().contains("16"));
    }

    #[test]
    fn supports_fudge_die() {
        let rng = StubRng::new(vec![0, 1, -1]);
        let result = stub_roll("3dF", rng).expect("roll should succeed");

        assert_eq!(result.total(), 0);
    }

    #[test]
    fn clamp_min_and_max() {
        let rng = StubRng::new(vec![1, 4, 6]);
        let result = stub_roll("3d6min3max5", rng).expect("roll should succeed");

        assert_eq!(result.total(), 12);
    }

    #[test]
    fn reroll_defaults_to_infinite_on_one() {
        let rng = StubRng::new(vec![1, 5, 2]);
        let result = stub_roll("2d6r", rng).expect("roll should succeed");

        assert_eq!(result.total(), 7);
    }

    #[test]
    fn reroll_with_count_and_condition() {
        let rng = StubRng::new(vec![1, 2, 4, 6]);
        let result = stub_roll("2d6r<=3times2", rng).expect("roll should succeed");

        assert_eq!(result.total(), 10);
    }

    #[test]
    fn drop_condition_and_keep_condition() {
        let drop_rng = StubRng::new(vec![6, 5, 4, 3]);
        let drop_result = stub_roll("4d6d>=5", drop_rng).expect("roll should succeed");
        assert_eq!(drop_result.total(), 7);

        let keep_rng = StubRng::new(vec![10, 12, 18, 3]);
        let keep_result = stub_roll("4d20k>=12", keep_rng).expect("roll should succeed");
        assert_eq!(keep_result.total(), 30);
    }

    #[test]
    fn keep_highest_count() {
        let rng = StubRng::new(vec![3, 6, 2, 5]);
        let result = stub_roll("4d6kh2", rng).expect("roll should succeed");
        assert_eq!(result.total(), 11);
    }

    #[test]
    fn compare_counts_matches() {
        let rng = StubRng::new(vec![6, 5, 2, 1]);
        let result = stub_roll("4d6c>=5", rng).expect("roll should succeed");

        assert_eq!(result.total(), 2);
    }

    #[test]
    fn sort_ascending_changes_display_order() {
        let rng = StubRng::new(vec![6, 2, 4]);
        let result = stub_roll("3d6sa", rng).expect("roll should succeed");

        assert!(result.to_string().contains("(2, 4, 6) 12"));
    }

    #[test]
    fn explode_defaults_to_one_extra_roll() {
        let rng = StubRng::new(vec![6, 4]);

        let result = stub_roll("1d6ex=6", rng).expect("roll should succeed");
        println!("{:?}", result);

        assert_eq!(result.total(), 10);
    }

    #[test]
    fn explode_respects_times_limit() {
        let rng = StubRng::new(vec![6, 6, 2]);
        let result = stub_roll("1d6ex=6times2", rng).expect("roll should succeed");

        assert_eq!(result.total(), 14);
    }

    #[test]
    fn supports_advantage_and_disadvantage() {
        let adv_rng = StubRng::new(vec![18, 5]);
        let adv = stub_roll("d20adv", adv_rng).expect("roll should succeed");
        assert_eq!(adv.total(), 18);

        let dis_rng = StubRng::new(vec![18, 5]);
        let dis = stub_roll("d20dis", dis_rng).expect("roll should succeed");
        assert_eq!(dis.total(), 5);
    }

    #[test]
    fn unary_minus_precedence_is_higher_than_multiplication() {
        let rng = StubRng::new(vec![]);
        let result = stub_roll("-2 * 3", rng).expect("parse should succeed");

        assert_eq!(result.total(), -6);
    }

    #[test]
    fn unique_modifier_returns_all_unique_values() {
        // Known bug repro: unique reroll logic can keep a duplicate and drop a unique roll.
        let rng = StubRng::new(vec![1, 1, 2, 3, 1, 6, 5, 3, 2, 1, 3, 4]);
        let result = stub_roll("6d6u", rng).expect("roll should succeed");

        match result {
            EvalResult::DiceRollGroup { rolls, .. } => {
                let mut kept = rolls
                    .into_iter()
                    .filter(|r| !r.dropped)
                    .map(|r| r.result)
                    .collect::<Vec<_>>();
                kept.sort_unstable();
                assert_eq!(kept, vec![1, 2, 3, 4, 5, 6]);
            }
            _ => panic!("expected dice roll group"),
        }
    }

    #[test]
    fn rejects_division_by_zero() {
        let rng = StubRng::new(vec![]);
        let error = stub_roll("1 / 0", rng).expect_err("roll should fail");
        assert!(error
            .to_string()
            .to_lowercase()
            .contains("division by zero"));
    }
}
