use std::process::ExitCode;

use crate::error::PeshResult;

pub struct Evaluator {}

impl Evaluator {
    pub fn new() -> Self {
        Evaluator {}
    }
    pub fn eval_raw(&self, command_raw: String) -> PeshResult<ExitCode> {
        todo!()
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}
