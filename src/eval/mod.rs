use std::process::ExitCode;

use crate::error::{EvaluatorError, PeshError, PeshResult};

pub struct Evaluator {}

impl Evaluator {
    pub fn new() -> Self {
        Evaluator {}
    }
    pub fn eval_raw(&self, command_raw: String) -> PeshResult<ExitCode> {
        Err(PeshError::Evaluator(
            command_raw,
            EvaluatorError::CommandNotFound,
        ))
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}
