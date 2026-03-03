use std::io;

use thiserror::Error;

pub type PeshResult<T> = std::result::Result<T, PeshError>;

#[derive(Error, Debug)]
pub enum PeshError {
    #[error("os error")]
    Os(#[from] io::Error),
    #[error("{0}: {1}")]
    Evaluator(String, EvaluatorError),
    #[error("Input Error: {0}")]
    Input(#[from] dialoguer::Error),
}

#[derive(Error, Debug)]
pub enum EvaluatorError {
    #[error("command not found")]
    CommandNotFound,
    #[error("input could not be parsed")]
    SplitError,
}
