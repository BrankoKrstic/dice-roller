use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::dsl::lexer::{Lexer, LexerError, Token, TokenKind};

#[derive(Debug, Error)]
pub enum ParserError {
    #[error("Unterminated expression")]
    UnterminatedExpression,
    #[error("Malformed expression")]
    MalformedExpression,
    #[error("Lexer error {error}")]
    LexerError { error: LexerError },
    #[error("Unexpected token found {token}")]
    UnexpectedToken { token: Token },
    #[error("Unexpected dice type {token}. Supported types are d4/d6/d8/d10/d12/d20/d%/dF")]
    UnexpectedDiceType { token: Token },
    #[error("Invalid Dice Modifiers: {message}")]
    InvalidModifiers { message: String },
}

impl From<LexerError> for ParserError {
    fn from(error: LexerError) -> Self {
        ParserError::LexerError { error }
    }
}

pub struct Parser<'input> {
    lexer: Lexer<'input>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiceKind {
    D4,
    D6,
    D8,
    D10,
    D12,
    D20,
    DPercentile,
    DFudge,
}

impl DiceKind {
    pub fn max_val(&self) -> u32 {
        match self {
            DiceKind::D4 => 4,
            DiceKind::D6 => 6,
            DiceKind::D8 => 8,
            DiceKind::D10 => 10,
            DiceKind::D12 => 12,
            DiceKind::D20 => 20,
            DiceKind::DPercentile => 100,
            DiceKind::DFudge => 1,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortOrder {
    Asc,
    Dsc,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiceModifier {
    Explode {
        condition: Condition,
        count: Option<u32>,
    },
    Keep {
        condition: Condition,
    },
    Sort(SortOrder),
    Reroll {
        times: u32,
        condition: Condition,
    },
    Drop {
        condition: Condition,
    },
    Count {
        condition: Option<Condition>,
    },
    Unique,
    Min(i32),
    Max(i32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModifierOp {
    Greater,
    Less,
    GreaterEqual,
    LessEqual,
    Equal,
    Lowest,
    Highest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Condition {
    pub(crate) target: u32,
    pub(crate) op: ModifierOp,
}

impl Condition {
    pub fn new(target: u32, op: ModifierOp) -> Self {
        Self { target, op }
    }
}

impl std::fmt::Display for Condition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let op = match self.op {
            ModifierOp::Greater => ">",
            ModifierOp::Less => "<",
            ModifierOp::GreaterEqual => ">=",
            ModifierOp::LessEqual => "<=",
            ModifierOp::Equal => "=",
            ModifierOp::Lowest => "l",
            ModifierOp::Highest => "h",
        };

        write!(f, "{op}{}", self.target)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dice {
    pub(crate) count: u32,
    pub(crate) kind: DiceKind,
    pub(crate) modifiers: Vec<DiceModifier>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOp {
    Negate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Ast {
    Number(u32),
    Dice(Dice),
    Unary {
        op: UnaryOp,
        ast: Box<Ast>,
    },
    Binary {
        op: BinaryOp,
        lhs: Box<Ast>,
        rhs: Box<Ast>,
    },
}

impl std::fmt::Display for Ast {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ast::Number(value) => write!(f, "{value}"),
            Ast::Dice(dice) => {
                write!(f, "{}{}", dice.count, display_dice_kind(dice.kind))?;
                for modifier in &dice.modifiers {
                    write!(f, "{}", display_dice_modifier(modifier))?;
                }
                Ok(())
            }
            Ast::Unary {
                op: UnaryOp::Negate,
                ast,
            } => write!(f, "-{}", display_wrapped_ast(ast)),
            Ast::Binary { op, lhs, rhs } => {
                let op = match op {
                    BinaryOp::Add => "+",
                    BinaryOp::Subtract => "-",
                    BinaryOp::Multiply => "*",
                    BinaryOp::Divide => "/",
                };

                write!(
                    f,
                    "{} {op} {}",
                    display_wrapped_ast(lhs),
                    display_wrapped_ast(rhs)
                )
            }
        }
    }
}

fn display_wrapped_ast(ast: &Ast) -> String {
    match ast {
        Ast::Binary { .. } => format!("({ast})"),
        _ => ast.to_string(),
    }
}

fn display_dice_kind(kind: DiceKind) -> &'static str {
    match kind {
        DiceKind::D4 => "d4",
        DiceKind::D6 => "d6",
        DiceKind::D8 => "d8",
        DiceKind::D10 => "d10",
        DiceKind::D12 => "d12",
        DiceKind::D20 => "d20",
        DiceKind::DPercentile => "d%",
        DiceKind::DFudge => "dF",
    }
}

fn display_dice_modifier(modifier: &DiceModifier) -> String {
    match modifier {
        DiceModifier::Explode { condition, count } => match count {
            Some(count) => format!("ex{condition}times{count}"),
            None => format!("ex{condition}"),
        },
        DiceModifier::Keep { condition } => format!("k{condition}"),
        DiceModifier::Sort(order) => match order {
            SortOrder::Asc => "sasc".to_string(),
            SortOrder::Dsc => "sdsc".to_string(),
        },
        DiceModifier::Reroll { times, condition } => format!("r{condition}times{times}"),
        DiceModifier::Drop { condition } => format!("d{condition}"),
        DiceModifier::Count { condition } => match condition {
            Some(condition) => format!("c{condition}"),
            None => "c".to_string(),
        },
        DiceModifier::Unique => "u".to_string(),
        DiceModifier::Min(value) => format!("min{value}"),
        DiceModifier::Max(value) => format!("max{value}"),
    }
}

type ParseResult = Result<Ast, ParserError>;

impl<'input> Parser<'input> {
    pub fn new(input: &'input str) -> Self {
        Self {
            lexer: Lexer::new(input),
        }
    }

    pub fn parse(&mut self) -> ParseResult {
        let result = self.parse_expr_within(0)?;
        if !self.lexer.is_done() {
            Err(ParserError::MalformedExpression)
        } else {
            Ok(result)
        }
    }

    fn peek_number(&mut self) -> Option<&u32> {
        match self.lexer.peek() {
            Some(Ok(Token {
                kind: TokenKind::Number(num),
                ..
            })) => Some(num),
            _ => None,
        }
    }

    fn expect_number(&mut self) -> Result<u32, ParserError> {
        match self.lexer.next() {
            Some(Ok(Token {
                kind: TokenKind::Number(num),
                ..
            })) => Ok(num),
            Some(Ok(token)) => Err(ParserError::UnexpectedToken { token }),
            Some(Err(err)) => Err(err)?,
            None => Err(ParserError::UnterminatedExpression),
        }
    }
    fn parse_times(&mut self) -> Result<Option<u32>, ParserError> {
        let Some(Ok(token)) = self.lexer.peek() else {
            return Ok(None);
        };
        if matches!(token.kind, TokenKind::Times) {
            self.lexer.next();
            return self.expect_number().map(Some);
        }
        Ok(None)
    }
    fn parse_condition(&mut self) -> Result<Option<Condition>, ParserError> {
        let Some(Ok(token)) = self.lexer.peek() else {
            return Ok(None);
        };

        let op = match token.kind {
            TokenKind::H => ModifierOp::Highest,
            TokenKind::L => ModifierOp::Lowest,
            TokenKind::GreaterEqual => ModifierOp::GreaterEqual,
            TokenKind::LessEqual => ModifierOp::LessEqual,
            TokenKind::Equal => ModifierOp::Equal,
            TokenKind::Greater => ModifierOp::Greater,
            TokenKind::Less => ModifierOp::Less,
            _ => return Ok(None),
        };
        self.lexer.next();
        let target = match op {
            ModifierOp::Highest | ModifierOp::Lowest => match self.peek_number() {
                Some(_) => self.expect_number()?,
                None => 1,
            },
            _ => self.expect_number()?,
        };
        Ok(Some(Condition::new(target, op)))
    }

    fn parse_dice(&mut self, count: u32) -> ParseResult {
        self.lexer.expect(TokenKind::D)?;
        let next = match self.lexer.next() {
            Some(Ok(token)) => token,
            Some(err) => err?,
            None => Err(ParserError::UnterminatedExpression)?,
        };
        let dice_kind = match next.kind {
            TokenKind::Number(num) if matches!(num, 4 | 6 | 8 | 10 | 12 | 20) => match num {
                4 => DiceKind::D4,
                6 => DiceKind::D6,
                8 => DiceKind::D8,
                10 => DiceKind::D10,
                12 => DiceKind::D12,
                20 => DiceKind::D20,
                _ => unreachable!(),
            },
            TokenKind::Percent => DiceKind::DPercentile,
            TokenKind::Fudge => DiceKind::DFudge,
            _ => Err(ParserError::UnexpectedDiceType { token: next })?,
        };
        let dice = self.parse_modifiers(count, dice_kind)?;
        Ok(Ast::Dice(dice))
    }

    fn parse_modifiers(&mut self, count: u32, dice: DiceKind) -> Result<Dice, ParserError> {
        let mut out = vec![];
        let mut has_adv = false;
        let mut has_dis = false;
        let mut has_u = false;
        while let Some(Ok(token)) = self.lexer.peek() {
            let modifier = match token.kind {
                TokenKind::D => {
                    self.lexer.next();
                    let condition = self
                        .parse_condition()?
                        .unwrap_or(Condition::new(1, ModifierOp::Lowest));
                    DiceModifier::Drop { condition }
                }
                TokenKind::Ex => {
                    self.lexer.next();
                    let condition = self
                        .parse_condition()?
                        .unwrap_or(Condition::new(dice.max_val(), ModifierOp::Equal));

                    let times = self.parse_times()?;

                    Self::validate_condition(dice, condition, count, times)?;

                    DiceModifier::Explode {
                        count: times,
                        condition,
                    }
                }
                TokenKind::K => {
                    self.lexer.next();
                    let condition = self.parse_condition()?.unwrap_or(Condition {
                        target: 1,
                        op: ModifierOp::Highest,
                    });
                    DiceModifier::Keep { condition }
                }
                TokenKind::R => {
                    self.lexer.next();

                    let condition = self.parse_condition()?.unwrap_or(Condition {
                        target: 1,
                        op: ModifierOp::Lowest,
                    });
                    let times = self.parse_times()?.unwrap_or(1);
                    DiceModifier::Reroll { condition, times }
                }
                TokenKind::U => {
                    has_u = true;
                    self.lexer.next();
                    let options = match dice {
                        DiceKind::DFudge => 3,
                        x => x.max_val(),
                    };
                    if count > options {
                        Err(ParserError::InvalidModifiers {
                            message: "More dice then there are possible unique results".to_string(),
                        })?;
                    }
                    DiceModifier::Unique
                }
                TokenKind::C => {
                    self.lexer.next();
                    let condition = self.parse_condition()?;
                    DiceModifier::Count { condition }
                }
                TokenKind::S => {
                    self.lexer.next();
                    DiceModifier::Sort(SortOrder::Dsc)
                }
                TokenKind::Sa => {
                    self.lexer.next();
                    DiceModifier::Sort(SortOrder::Asc)
                }
                TokenKind::Min => {
                    self.lexer.next();
                    let min = self.expect_number()?;
                    DiceModifier::Min(min as i32)
                }
                TokenKind::Max => {
                    self.lexer.next();
                    let max = self.expect_number()?;
                    DiceModifier::Max(max as i32)
                }
                TokenKind::Adv => {
                    has_adv = true;
                    self.lexer.next();
                    if count != 1 {
                        Err(ParserError::InvalidModifiers {
                            message: "adv/dis are only valid on single-die rolls".to_string(),
                        })?;
                    }
                    DiceModifier::Drop {
                        condition: Condition::new(1, ModifierOp::Lowest),
                    }
                }
                TokenKind::Dis => {
                    has_dis = true;
                    self.lexer.next();
                    if count != 1 {
                        Err(ParserError::InvalidModifiers {
                            message: "adv/dis are only valid on single-die rolls".to_string(),
                        })?;
                    }
                    DiceModifier::Drop {
                        condition: Condition::new(1, ModifierOp::Highest),
                    }
                }
                _ => break,
            };
            out.push(modifier)
        }

        Self::validate_modifiers(dice, &out, has_adv, has_dis, has_u)?;

        Ok(Dice {
            count: if has_adv || has_dis { 2 } else { count },
            kind: dice,
            modifiers: out,
        })
    }
    fn validate_condition(
        dice_kind: DiceKind,
        condition: Condition,
        dice_count: u32,
        times: Option<u32>,
    ) -> Result<(), ParserError> {
        let max_result = dice_kind.max_val();
        if times.is_some() {
            return Ok(());
        }
        let target = condition.target;

        let is_valid = match condition.op {
            ModifierOp::Greater => target < max_result,
            ModifierOp::Less => target > 1,
            ModifierOp::GreaterEqual => target <= max_result,
            ModifierOp::LessEqual => target > 0,
            ModifierOp::Equal => true,
            ModifierOp::Lowest => target < dice_count,
            ModifierOp::Highest => target < dice_count,
        };

        if !is_valid {
            Err(ParserError::InvalidModifiers {
				message: "Condition would lead to endless rerolling/exploding. Please modify the condition or cap rerolls by using the times[num] modifier".to_string()
			})?;
        }

        Ok(())
    }
    fn validate_modifiers(
        dice: DiceKind,
        mods: &[DiceModifier],
        has_adv: bool,
        has_dis: bool,
        has_u: bool,
    ) -> Result<(), ParserError> {
        if (has_adv || has_dis || has_u) && mods.len() > 1 {
            return Err(ParserError::InvalidModifiers { message: "Dice modifiers adv, dis, and u are unique and can't be combined with other modifiers".to_string() });
        }

        if (has_adv || has_dis) && !matches!(dice, DiceKind::D20) {
            return Err(ParserError::InvalidModifiers {
                message: "Adv and dis modifiers can only be used with d20".to_string(),
            });
        }

        Ok(())
    }
    fn parse_expr_within(&mut self, min_bp: u32) -> ParseResult {
        let token = match self.lexer.next() {
            Some(Ok(token)) => token,
            None => Err(ParserError::UnterminatedExpression)?,
            Some(Err(err)) => Err(err)?,
        };

        let mut lhs = match token.kind {
            TokenKind::Number(num) => {
                if let Some(Ok(Token {
                    kind: TokenKind::D, ..
                })) = self.lexer.peek()
                {
                    self.parse_dice(num)?
                } else {
                    Ast::Number(num)
                }
            }
            TokenKind::D => {
                self.lexer.give_back(token);
                self.parse_dice(1)?
            }
            TokenKind::Minus => {
                let op: UnaryOp = UnaryOp::Negate;
                let ((), prefix_bp) = prefix_binding_power(op);
                let ast = self.parse_expr_within(prefix_bp)?;
                Ast::Unary {
                    op,
                    ast: ast.into(),
                }
            }
            TokenKind::LeftParen => {
                let inner = self.parse_expr_within(0)?;
                self.lexer.expect(TokenKind::RightParen)?;
                inner
            }
            _ => Err(ParserError::UnexpectedToken { token })?,
        };

        loop {
            let op = match self.lexer.peek() {
                Some(Ok(token)) => token,
                Some(Err(_)) => Err(self
                    .lexer
                    .next()
                    .expect("checked option")
                    .expect_err("checked err"))?,
                None => break,
            };

            let op = match op.kind {
                TokenKind::RightParen => break,
                TokenKind::Plus => BinaryOp::Add,
                TokenKind::Minus => BinaryOp::Subtract,
                TokenKind::Star => BinaryOp::Multiply,
                TokenKind::Slash => BinaryOp::Divide,
                _ => Err(ParserError::UnexpectedToken { token: *op })?,
            };
            let (left_bp, right_bp) = infix_binding_power(op);

            if left_bp < min_bp {
                break;
            }
            self.lexer.next();
            let rhs = self.parse_expr_within(right_bp)?;
            lhs = Ast::Binary {
                op,
                lhs: lhs.into(),
                rhs: rhs.into(),
            };
        }
        Ok(lhs)
    }
}

fn prefix_binding_power(op: UnaryOp) -> ((), u32) {
    match op {
        UnaryOp::Negate => ((), 5),
    }
}

fn infix_binding_power(op: BinaryOp) -> (u32, u32) {
    match op {
        BinaryOp::Add | BinaryOp::Subtract => (1, 2),
        BinaryOp::Multiply | BinaryOp::Divide => (3, 4),
    }
}

#[cfg(test)]
mod tests {
    use crate::dsl::parser::{
        Ast, BinaryOp, Condition, DiceKind, DiceModifier, ModifierOp, Parser, SortOrder, UnaryOp,
    };

    #[test]
    fn parses_precedence() {
        let expr = Parser::new("1 + 2 * 3")
            .parse()
            .expect("parse should succeed");

        match expr {
            Ast::Binary {
                lhs,
                op: BinaryOp::Add,
                rhs,
            } => {
                assert!(matches!(*lhs, Ast::Number(1)));
                assert!(matches!(
                    *rhs,
                    Ast::Binary {
                        op: BinaryOp::Multiply,
                        ..
                    }
                ));
            }
            _ => panic!("expected top-level addition"),
        }
    }

    #[test]
    fn unary_minus_binds_tighter_than_multiplication() {
        let expr = Parser::new("-2 * 3").parse().expect("parse should succeed");

        match expr {
            Ast::Binary {
                op: BinaryOp::Multiply,
                lhs,
                rhs,
            } => {
                assert!(matches!(
                    *lhs,
                    Ast::Unary {
                        op: UnaryOp::Negate,
                        ..
                    }
                ));
                assert!(matches!(*rhs, Ast::Number(3)));
            }
            _ => panic!("expected multiplication expression"),
        }
    }

    #[test]
    fn adv_rewrites_to_drop_lowest() {
        let expr = Parser::new("d20adv").parse().expect("parse should succeed");

        match expr {
            Ast::Dice(dice) => {
                assert_eq!(dice.count, 2);
                assert_eq!(dice.kind, DiceKind::D20);
                assert_eq!(
                    dice.modifiers,
                    vec![DiceModifier::Drop {
                        condition: Condition::new(1, ModifierOp::Lowest),
                    }]
                );
            }
            _ => panic!("expected dice expression"),
        }
    }

    #[test]
    fn parses_fudge_die() {
        let expr = Parser::new("4dF").parse().expect("parse should succeed");
        match expr {
            Ast::Dice(dice) => {
                assert_eq!(dice.kind, DiceKind::DFudge);
                assert_eq!(dice.count, 4);
            }
            _ => panic!("expected dice expression"),
        }
    }

    #[test]
    fn parses_new_modifiers() {
        let expr = Parser::new("4d6r<=3times2kh2d>=5c>=6smin2max5")
            .parse()
            .expect("parse should succeed");

        match expr {
            Ast::Dice(dice) => {
                assert_eq!(
                    dice.modifiers,
                    vec![
                        DiceModifier::Reroll {
                            times: 2,
                            condition: Condition::new(3, ModifierOp::LessEqual),
                        },
                        DiceModifier::Keep {
                            condition: Condition::new(2, ModifierOp::Highest),
                        },
                        DiceModifier::Drop {
                            condition: Condition::new(5, ModifierOp::GreaterEqual),
                        },
                        DiceModifier::Count {
                            condition: Some(Condition {
                                target: 6,
                                op: ModifierOp::GreaterEqual
                            })
                        },
                        DiceModifier::Sort(SortOrder::Dsc),
                        DiceModifier::Min(2),
                        DiceModifier::Max(5),
                    ]
                );
            }
            _ => panic!("expected dice expression"),
        }
    }

    #[test]
    fn ex_defaults_to_unlimited_extra_rolls() {
        let expr = Parser::new("1d6ex>=6times3")
            .parse()
            .expect("parse should succeed");

        match expr {
            Ast::Dice(dice) => {
                assert_eq!(
                    dice.modifiers,
                    vec![DiceModifier::Explode {
                        count: Some(3),
                        condition: Condition::new(6, ModifierOp::GreaterEqual)
                    }]
                );
            }
            _ => panic!("expected dice expression"),
        }
    }

    #[test]
    fn dl_and_dh_default_to_one() {
        let expr = Parser::new("4d6dldh")
            .parse()
            .expect("parse should succeed");

        match expr {
            Ast::Dice(dice) => {
                assert_eq!(
                    dice.modifiers,
                    vec![
                        DiceModifier::Drop {
                            condition: Condition::new(1, ModifierOp::Lowest)
                        },
                        DiceModifier::Drop {
                            condition: Condition::new(1, ModifierOp::Highest)
                        }
                    ]
                );
            }
            _ => panic!("expected dice expression"),
        }
    }

    #[test]
    fn rejects_adv_for_non_d20() {
        let error = Parser::new("d6adv").parse().expect_err("parse should fail");
        assert!(
            error
                .to_string()
                .contains("Adv and dis modifiers can only be used with d20")
        );
    }

    #[test]
    fn rejects_adv_for_multi_die_terms() {
        let error = Parser::new("2d20adv")
            .parse()
            .expect_err("parse should fail");
        assert!(
            error
                .to_string()
                .contains("adv/dis are only valid on single-die rolls")
        );
    }

    #[test]
    fn rejects_missing_closing_parenthesis() {
        let error = Parser::new("(1 + 2")
            .parse()
            .expect_err("parse should fail");
        assert!(error.to_string().contains("Unexpected end of expression"));
    }

    #[test]
    fn disallows_unmatched_closing_brace() {
        let error = Parser::new("1)+2").parse().expect_err("parse should fail");

        assert!(error.to_string().contains("Malformed expression"));
    }

    #[test]
    fn rejects_unique_when_not_enough_outcomes() {
        let error = Parser::new("4dFu").parse().expect_err("parse should fail");

        assert!(
            error
                .to_string()
                .contains("More dice then there are possible unique result")
        );
    }
}
