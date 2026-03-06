use std::io;

use thiserror::Error;

pub type PeshResult<T> = std::result::Result<T, PeshError>;

#[derive(Error, Debug)]
pub enum PeshError {
    #[error(transparent)]
    Os(#[from] io::Error),
    #[error(transparent)]
    Evaluator(#[from] EvaluatorError),
    #[error("Input Error: {0}")]
    Input(#[from] dialoguer::Error),
}

#[derive(Error, Debug)]
pub enum EvaluatorError {
    #[error("not found")]
    CommandNotFound,
    #[error("input could not be parsed")]
    SplitError,
    #[error("wrong number of arguments")]
    WrongNumberOfArguments(u8),
}
