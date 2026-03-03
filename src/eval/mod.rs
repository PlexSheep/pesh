pub mod instruction;

use std::process::ExitCode;

use crate::error::{EvaluatorError, PeshError, PeshResult};

pub struct Evaluator {}

impl Evaluator {
    pub fn new() -> Self {
        Evaluator {}
    }

    #[inline]
    pub fn eval_raw(&self, command_raw: &str) -> PeshResult<ExitCode> {
        self.eval(&self.split(command_raw)?)
    }

    pub fn split(&self, command_raw: &str) -> PeshResult<Vec<String>> {
        match shlex::split(command_raw) {
            Some(parts) => Ok(parts),
            None => Err(PeshError::Evaluator(
                command_raw.to_string(),
                EvaluatorError::SplitError,
            )),
        }
    }

    pub fn eval(&self, command: &[String]) -> PeshResult<ExitCode> {
        // TODO: make sure that we have at least one element
        Err(PeshError::Evaluator(
            command[0].to_string(),
            EvaluatorError::CommandNotFound,
        ))
    }
}

impl Default for Evaluator {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::eval::Evaluator;

    #[test]
    fn eval_split() {
        let e = Evaluator::default();
        assert_eq!(e.split("hello world").unwrap(), ["hello", "world"]);
        assert_eq!(
            e.split("hello world 19193139-asfjkagsjiju==??").unwrap(),
            ["hello", "world", "19193139-asfjkagsjiju==??"]
        );
        assert_eq!(
            e.split("hello \"world of love\"").unwrap(),
            ["hello", "world of love"]
        );
        assert_eq!(
            e.split("hello \"world \\\"of love\"").unwrap(),
            ["hello", "world \"of love"]
        );
        assert_eq!(
            e.split("hello \"world's boom \\\"of love\"").unwrap(),
            ["hello", "world's boom \"of love"]
        );
    }
}
