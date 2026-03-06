pub mod command;

use std::{
    fs,
    io::{self, Stderr},
    path::PathBuf,
};

use nix::unistd::AccessFlags;

use crate::{
    error::{EvaluatorError, PeshError, PeshResult},
    eval::command::{CommandTask, composite::Command},
};

pub struct Evaluator {}

impl Evaluator {
    pub fn new() -> Self {
        Evaluator {}
    }

    #[inline]
    pub fn eval_raw(&self, command_raw: &str) -> PeshResult<Command> {
        let normalized: String = command_raw
            .to_string()
            .replace("1>", " 1> ")
            .replace("2>", " 2> ")
            .replace(">", " 1> ");

        self.eval_command(&self.split(&normalized)?)
    }

    fn split(&self, command_raw: &str) -> PeshResult<Vec<String>> {
        match shlex::split(command_raw) {
            Some(parts) => Ok(parts),
            None => Err(PeshError::Evaluator(EvaluatorError::SplitError))?,
        }
    }

    pub fn eval_task(&self, command: &[String]) -> PeshResult<CommandTask> {
        assert!(!command.is_empty());

        CommandTask::try_from(command).map_err(PeshError::from)
    }

    pub fn eval_command(&self, parts: &[String]) -> PeshResult<Command> {
        #[derive(Default, Debug, Copy, Clone)]
        enum ParseState {
            #[default]
            Command,
            RedirStdout,
            RedirStderr,
        }

        let mut pstate = ParseState::default();
        let mut stdout_path = None;
        let mut stderr_path = None;
        let mut argv = Vec::new();

        for subpart in parts {
            match pstate {
                ParseState::Command => {
                    if subpart == "1>" {
                        pstate = ParseState::RedirStdout
                    } else if subpart == "2>" {
                        pstate = ParseState::RedirStderr
                    } else if subpart == ";" {
                        continue;
                    } else {
                        argv.push(subpart.to_owned());
                    }
                }
                ParseState::RedirStdout => {
                    stdout_path = Some((subpart).into());
                    pstate = ParseState::Command;
                }
                ParseState::RedirStderr => {
                    stderr_path = Some((subpart).into());
                    pstate = ParseState::Command;
                }
            }
        }
        let ct = self.eval_task(&argv)?;

        let cc = Command::new(ct)
            .with_stdout_to(stdout_path)
            .with_stderr_to(stderr_path);
        Ok(cc)
    }
}

pub fn get_home() -> PathBuf {
    std::env::home_dir().unwrap_or("/".into())
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

            if let Err(_e) = nix::unistd::access(&ent_path, AccessFlags::X_OK) {
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
            locate_executable("/usr/bin:/usr/sbin", "bash").unwrap(),
            Some("/usr/bin/bash".into())
        )
    }
}
