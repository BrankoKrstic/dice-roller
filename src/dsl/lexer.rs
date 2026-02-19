use thiserror::Error;

#[derive(Debug, Error)]
pub enum RollError {
    #[error("expression is empty")]
    EmptyExpression,
    #[error("expression is too long: {actual} (max {max})")]
    ExpressionTooLong { max: usize, actual: usize },
    #[error("lex error at {span}: {message}")]
    Lex { message: String, span: Span },
    #[error("parse error at {span}: {message}")]
    Parse { message: String, span: Span },
    #[error("evaluation error: {0}")]
    Eval(String),
}

#[derive(Debug, PartialEq, Eq)]
pub enum TokenKind {
    Number(i32),
    Dice,
    Percent,
    Fudge,
    Plus,
    Minus,
    Star,
    Slash,
    LeftParen,
    RightParen,
    GreaterEqual,
    LessEqual,
    Equal,
    Greater,
    Less,
    Ex,
    Times,
    Dl,
    Dh,
    Kh,
    Kl,
    K,
    R,
    U,
    C,
    S,
    Sa,
    Min,
    Max,
    Adv,
    Dis,
}

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number(num) => write!(f, "{}", num),
            Self::Dice => write!(f, "d"),
            Self::Percent => write!(f, "%"),
            Self::Fudge => write!(f, "F"),
            Self::Plus => write!(f, "+"),
            Self::Minus => write!(f, "-"),
            Self::Star => write!(f, "*"),
            Self::Slash => write!(f, "/"),
            Self::LeftParen => write!(f, "("),
            Self::RightParen => write!(f, ")"),
            Self::GreaterEqual => write!(f, ">="),
            Self::LessEqual => write!(f, "<="),
            Self::Equal => write!(f, "="),
            Self::Greater => write!(f, ">"),
            Self::Less => write!(f, "<"),
            Self::Ex => write!(f, "ex"),
            Self::Times => write!(f, "times"),
            Self::Dl => write!(f, "dl"),
            Self::Dh => write!(f, "dh"),
            Self::Kh => write!(f, "kh"),
            Self::Kl => write!(f, "kl"),
            Self::K => write!(f, "k"),
            Self::R => write!(f, "r"),
            Self::U => write!(f, "u"),
            Self::C => write!(f, "c"),
            Self::S => write!(f, "s"),
            Self::Sa => write!(f, "sa"),
            Self::Min => write!(f, "min"),
            Self::Max => write!(f, "max"),
            Self::Adv => write!(f, "adv"),
            Self::Dis => write!(f, "dis"),
        }
    }
}

#[derive(Debug)]
pub struct Span {
    start: usize,
    end: usize,
}

impl Span {
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

#[derive(Debug)]
pub struct Token {
    span: Span,
    kind: TokenKind,
}

pub struct Lexer<'input> {
    whole: &'input str,
    rest: &'input str,
}

impl<'input> Lexer<'input> {
    pub fn is_done(&self) -> bool {
        self.rest.is_empty()
    }
    pub fn new(input: &'input str) -> Self {
        Self {
            whole: input,
            rest: input,
        }
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Result<Token, RollError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let mut chars = self.rest.chars();
            let c = chars.next()?;
            let c_at = self.whole.len() - self.rest.len();

            self.rest = chars.as_str();
            enum Started {
                Number,
                A,
                D,
                E,
                K,
                S,
                M,
                T,
                IfEqualElse(TokenKind, TokenKind),
            }
            let just = move |kind: TokenKind| {
                Some(Ok(Token {
                    kind,
                    span: Span {
                        start: c_at,
                        end: c_at + 1,
                    },
                }))
            };

            let started = match c.to_ascii_lowercase() {
                '(' => return just(TokenKind::LeftParen),
                ')' => return just(TokenKind::RightParen),
                '+' => return just(TokenKind::Plus),
                '-' => return just(TokenKind::Minus),
                '*' => return just(TokenKind::Star),
                '/' => return just(TokenKind::Slash),
                '=' => return just(TokenKind::Equal),
                '%' => return just(TokenKind::Percent),
                'r' => return just(TokenKind::R),
                'c' => return just(TokenKind::C),
                'u' => return just(TokenKind::U),
                'f' => return just(TokenKind::Fudge),
                '<' => Started::IfEqualElse(TokenKind::LessEqual, TokenKind::Less),
                '>' => Started::IfEqualElse(TokenKind::GreaterEqual, TokenKind::Greater),
                'a' => Started::A,
                'd' => Started::D,
                'e' => Started::E,
                'k' => Started::K,
                's' => Started::S,
                'm' => Started::M,
                't' => Started::T,
                x if x.is_numeric() => Started::Number,
                x if x.is_whitespace() => continue,
                x => {
                    return Some(Err(RollError::Lex {
                        message: format!("unknown identifier {x}"),
                        span: Span::new(c_at, c_at + 1),
                    }));
                }
            };

            let mut check_start = |expected: &str, on_success: TokenKind| {
                if self.rest[..expected.len()].eq_ignore_ascii_case(expected) {
                    self.rest = &self.rest[expected.len()..];
                    Some(Ok(Token {
                        kind: on_success,
                        span: Span::new(c_at, c_at + expected.len()),
                    }))
                } else {
                    None
                }
            };
            let result = match started {
                Started::Number => {
                    let first_non_digit = self
                        .rest
                        .find(|c: char| !c.is_numeric())
                        .unwrap_or(self.rest.len());
                    let c_to = c_at + first_non_digit + 1;
                    let digits = &self.whole[c_at..c_to];
                    let n = match digits.parse() {
                        Ok(num) => num,
                        Err(_) => {
                            return Some(Err(RollError::Lex {
                                message: format!("Failed to parse {digits} as i32"),
                                span: Span::new(c_at, c_to),
                            }));
                        }
                    };
                    self.rest = &self.rest[first_non_digit..];
                    Some(Ok(Token {
                        kind: TokenKind::Number(n),
                        span: Span::new(c_at, c_to),
                    }))
                }
                Started::A => check_start("dv", TokenKind::Adv),
                Started::D => check_start("is", TokenKind::Dis)
                    .or_else(|| check_start("l", TokenKind::Dl))
                    .or_else(|| check_start("h", TokenKind::Dh))
                    .or(just(TokenKind::Dice)),
                Started::E => check_start("x", TokenKind::Ex),
                Started::K => check_start("h", TokenKind::Kh)
                    .or_else(|| check_start("l", TokenKind::Kl))
                    .or(just(TokenKind::K)),
                Started::S => check_start("a", TokenKind::Sa).or(just(TokenKind::S)),
                Started::M => {
                    check_start("in", TokenKind::Min).or_else(|| check_start("ax", TokenKind::Max))
                }
                Started::IfEqualElse(lhs, rhs) => check_start("=", lhs).or(just(rhs)),
                Started::T => check_start("imes", TokenKind::Times),
            };
            break result.or_else(|| {
                Some(Err(RollError::Lex {
                    message: format!("unknown identifier {}", &self.whole[c_at..c_at + 2]),
                    span: Span::new(c_at, c_at + 2),
                }))
            });
        }
    }
}

pub fn lex(input: &str) -> Result<Vec<Token>, RollError> {
    Lexer::new(input).collect()
}

#[cfg(test)]
mod tests {
    use super::{lex, TokenKind};

    #[test]
    fn lexes_basic_expression() {
        let tokens = lex("2d10 + 1d6 + 5").expect("lex should succeed");

        let kinds = tokens
            .into_iter()
            .map(|token| token.kind)
            .collect::<Vec<_>>();

        assert_eq!(
            kinds,
            vec![
                TokenKind::Number(2),
                TokenKind::Dice,
                TokenKind::Number(10),
                TokenKind::Plus,
                TokenKind::Number(1),
                TokenKind::Dice,
                TokenKind::Number(6),
                TokenKind::Plus,
                TokenKind::Number(5),
            ]
        );
    }

    #[test]
    fn lexes_stacked_modifiers() {
        let tokens = lex("1d6ex6times2dl2dhadvdis").expect("lex should succeed");

        let kinds = tokens
            .into_iter()
            .map(|token| token.kind)
            .collect::<Vec<_>>();

        assert_eq!(
            kinds,
            vec![
                TokenKind::Number(1),
                TokenKind::Dice,
                TokenKind::Number(6),
                TokenKind::Ex,
                TokenKind::Number(6),
                TokenKind::Times,
                TokenKind::Number(2),
                TokenKind::Dl,
                TokenKind::Number(2),
                TokenKind::Dh,
                TokenKind::Adv,
                TokenKind::Dis,
            ]
        );
    }

    #[test]
    fn lexes_new_modifiers_and_conditions() {
        let tokens = lex("4dFmin0max1r2<=0k>=0c=1sad>=1").expect("lex should succeed");

        let kinds = tokens
            .into_iter()
            .map(|token| token.kind)
            .collect::<Vec<_>>();

        assert_eq!(
            kinds,
            vec![
                TokenKind::Number(4),
                TokenKind::Dice,
                TokenKind::Fudge,
                TokenKind::Min,
                TokenKind::Number(0),
                TokenKind::Max,
                TokenKind::Number(1),
                TokenKind::R,
                TokenKind::Number(2),
                TokenKind::LessEqual,
                TokenKind::Number(0),
                TokenKind::K,
                TokenKind::GreaterEqual,
                TokenKind::Number(0),
                TokenKind::C,
                TokenKind::Equal,
                TokenKind::Number(1),
                TokenKind::Sa,
                TokenKind::Dice,
                TokenKind::GreaterEqual,
                TokenKind::Number(1),
            ]
        );
    }

    #[test]
    fn rejects_unknown_identifier() {
        let error = lex("1d6foo").expect_err("lex should fail");
        assert!(error.to_string().contains("unknown identifier"));
    }
}
