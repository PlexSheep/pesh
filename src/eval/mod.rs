pub mod command;

use std::process::ExitCode;

use crate::{
    error::{EvaluatorError, PeshError, PeshResult},
    eval::command::Command,
};

pub struct Evaluator {}

impl Evaluator {
    pub fn new() -> Self {
        Evaluator {}
    }

    #[inline]
    pub fn eval_raw(&self, command_raw: &str) -> PeshResult<Command> {
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

    pub fn eval(&self, command: &[String]) -> PeshResult<Command> {
        assert!(!command.is_empty());

        Command::try_from(command).map_err(|err| PeshError::Evaluator(command[0].to_string(), err))
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
