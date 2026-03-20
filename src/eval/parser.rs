use std::{
    fs,
    io::{self},
    path::PathBuf,
};

use crate::{
    error::{EvaluatorError, PeshError, PeshResult},
    eval::command::{CommandTask, composite::Command},
};

pub fn split(command_raw: &str) -> PeshResult<Vec<String>> {
    match shlex::split(command_raw) {
        Some(parts) => Ok(parts),
        None => Err(PeshError::Evaluator(EvaluatorError::SplitError))?,
    }
}
