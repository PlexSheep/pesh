pub mod command;

use std::{fs, io, path::PathBuf};

use nix::unistd::AccessFlags;

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

pub fn locate_executable(path_raw_env: &str, executable: &str) -> io::Result<Option<PathBuf>> {
    let mut path: PathBuf;
    let mut path_meta;
    for path_raw in path_raw_env.split(":") {
        path = path_raw.into();
        match path.metadata() {
            Ok(m) => path_meta = m,
            Err(_) => continue,
        }
        if !path_meta.is_dir() {
            continue;
        }

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let ent_path = entry.path();

            if ent_path.is_dir() {
                continue;
            }

            if ent_path.file_name().expect("no file name") != executable {
                continue;
            }

            if let Err(e) = nix::unistd::access(&ent_path, AccessFlags::X_OK) {
                continue;
            }

            return Ok(Some(ent_path));
        }
    }
    Ok(None)
}

impl Default for Evaluator {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn eval_locate_executable() {
        assert_eq!(
            locate_executable("/usr/bin:/usr/sbin", "bash"),
            Some("/usr/bin/bash".into())
        )
    }
}
