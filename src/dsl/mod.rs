use thiserror::Error;

use crate::dsl::{
    interpreter::{CryptoDiceRng, EvalResult, InterpreterError},
    lexer::LexerError,
    parser::ParserError,
};

pub mod interpreter;
pub mod lexer;
pub mod parser;

impl From<LexerError> for RollError {
    fn from(error: LexerError) -> Self {
        Self::Lex { error }
    }
}

impl From<ParserError> for RollError {
    fn from(value: ParserError) -> Self {
        match value {
            ParserError::LexerError { error } => error.into(),
            err => Self::Parse { error: err },
        }
    }
}

impl From<InterpreterError> for RollError {
    fn from(value: InterpreterError) -> Self {
        Self::Eval { error: value }
    }
}

#[derive(Debug, Error)]
pub enum RollError {
    #[error("Expression is empty")]
    EmptyExpression,
    #[error("Expression is too long: {actual} (max {max})")]
    ExpressionTooLong { max: usize, actual: usize },
    #[error("Lex error {error}")]
    Lex { error: LexerError },
    #[error("Parse error {error}")]
    Parse { error: ParserError },
    #[error("Runtime error: {error}")]
    Eval { error: InterpreterError },
}

pub fn parse_and_roll(input: &str) -> Result<EvalResult, RollError> {
    let mut parser = parser::Parser::new(input);
    let ast = parser.parse()?;
    let mut runtime = interpreter::Interpreter::new(CryptoDiceRng::new());
    Ok(runtime.eval_ast(&ast)?)
}
