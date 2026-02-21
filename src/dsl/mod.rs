use thiserror::Error;

use crate::dsl::lexer::LexerError;

pub mod interpreter;
pub mod lexer;
pub mod parser;

impl From<LexerError> for RollError {
    fn from(error: LexerError) -> Self {
        Self::Lex { error }
    }
}

#[derive(Debug, Error)]
pub enum RollError {
    #[error("expression is empty")]
    EmptyExpression,
    #[error("expression is too long: {actual} (max {max})")]
    ExpressionTooLong { max: usize, actual: usize },
    #[error("lex error {error}")]
    Lex { error: LexerError },
    #[error("evaluation error: {0}")]
    Eval(String),
}
