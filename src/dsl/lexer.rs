use std::fmt::Display;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum LexerError {
    #[error("Failed to parse {token} at {span}")]
    ParseError { token: String, span: Span },
    #[error("Unknown identifier {ident} at {span}")]
    UnknownIdentifier { ident: String, span: Span },
    #[error("Unexpected token {} at {}", token.kind, token.span)]
    UnexpectedToken { token: Token },
    #[error("Unexpected end of expression")]
    UnexpectedEof,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Number(u32),
    D,
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
    H,
    L,
    Ex,
    Times,
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
            Self::D => write!(f, "d"),
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
            Self::H => write!(f, "h"),
            Self::L => write!(f, "l"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
pub struct Token {
    span: Span,
    pub(crate) kind: TokenKind,
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind)
    }
}

pub struct Lexer<'input> {
    whole: &'input str,
    rest: &'input str,
    peeked: Option<Result<Token, LexerError>>,
}

impl<'input> Lexer<'input> {
    pub fn is_done(&self) -> bool {
        self.rest.is_empty()
    }
    pub fn new(input: &'input str) -> Self {
        Self {
            whole: input,
            rest: input,
            peeked: None,
        }
    }
    pub fn peek(&mut self) -> Option<&Result<Token, LexerError>> {
        if self.peeked.is_some() {
            return self.peeked.as_ref();
        }
        self.peeked = self.next();
        self.peeked.as_ref()
    }
    pub fn give_back(&mut self, token: Token) {
        debug_assert!(self.peeked.is_none());
        self.peeked = Some(Ok(token));
    }
    pub fn expect(&mut self, kind: TokenKind) -> Result<Token, LexerError> {
        self.expect_where(|next_kind| next_kind == &kind)
    }
    pub fn expect_where(&mut self, cond: impl Fn(&TokenKind) -> bool) -> Result<Token, LexerError> {
        match self.next() {
            Some(Ok(token)) if cond(&token.kind) => Ok(token),
            Some(Ok(token)) => Err(LexerError::UnexpectedToken { token }),
            Some(err) => err,
            None => Err(LexerError::UnexpectedEof),
        }
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Result<Token, LexerError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.peeked.is_some() {
            return self.peeked.take();
        }
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
                'l' => return just(TokenKind::L),
                'h' => return just(TokenKind::H),
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
                    return Some(Err(LexerError::UnknownIdentifier {
                        span: Span::new(c_at, c_at + 1),
                        ident: x.to_string(),
                    }));
                }
            };

            let mut check_start = |expected: &str, on_success: TokenKind| {
                if self.rest.len() >= expected.len()
                    && self.rest[..expected.len()].eq_ignore_ascii_case(expected)
                {
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
                            return Some(Err(LexerError::ParseError {
                                token: digits.to_string(),
                                span: Span::new(c_at, c_to),
                            }))
                        }
                    };
                    self.rest = &self.rest[first_non_digit..];
                    Some(Ok(Token {
                        kind: TokenKind::Number(n),
                        span: Span::new(c_at, c_to),
                    }))
                }
                Started::A => check_start("dv", TokenKind::Adv),
                Started::D => check_start("is", TokenKind::Dis).or(just(TokenKind::D)),
                Started::E => check_start("x", TokenKind::Ex),
                Started::K => just(TokenKind::K),
                Started::S => check_start("a", TokenKind::Sa).or(just(TokenKind::S)),
                Started::M => {
                    check_start("in", TokenKind::Min).or_else(|| check_start("ax", TokenKind::Max))
                }
                Started::IfEqualElse(lhs, rhs) => check_start("=", lhs).or(just(rhs)),
                Started::T => check_start("imes", TokenKind::Times),
            };
            break result.or_else(|| {
                let max_idx = (c_at + 2).min(self.whole.len());
                Some(Err(LexerError::UnknownIdentifier {
                    ident: self.whole[c_at..max_idx].to_string(),
                    span: Span::new(c_at, max_idx),
                }))
            });
        }
    }
}

pub fn lex(input: &str) -> Result<Vec<Token>, LexerError> {
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
                TokenKind::D,
                TokenKind::Number(10),
                TokenKind::Plus,
                TokenKind::Number(1),
                TokenKind::D,
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
                TokenKind::D,
                TokenKind::Number(6),
                TokenKind::Ex,
                TokenKind::Number(6),
                TokenKind::Times,
                TokenKind::Number(2),
                TokenKind::D,
                TokenKind::L,
                TokenKind::Number(2),
                TokenKind::D,
                TokenKind::H,
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
                TokenKind::D,
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
                TokenKind::D,
                TokenKind::GreaterEqual,
                TokenKind::Number(1),
            ]
        );
    }

    #[test]
    fn rejects_unknown_identifier() {
        let error = lex("1d6foo").expect_err("lex should fail");
        assert!(error
            .to_string()
            .to_lowercase()
            .contains("unknown identifier"));
    }
}
