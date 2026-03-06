pub mod command;

use std::{
    fs,
    io::{self, Stderr},
    path::PathBuf,
};

use nix::unistd::AccessFlags;

use crate::{
    error::{EvaluatorError, PeshError, PeshResult},
    eval::command::{Command, CompositeCommand},
};

pub struct Evaluator {}

impl Evaluator {
    pub fn new() -> Self {
        Evaluator {}
    }

    #[inline]
    pub fn eval_raw(&self, command_raw: &str) -> PeshResult<CompositeCommand> {
        let normalized: String = command_raw
            .to_string()
            .replace(";", " ; ")
            .replace("1>", " 1> ")
            .replace("2>", " 2> ")
            .replace(">", " 1> ")
            .replace("&&", " && ");

        self.eval_composite(&self.split(&normalized)?)
    }

    fn split(&self, command_raw: &str) -> PeshResult<Vec<String>> {
        match shlex::split(command_raw) {
            Some(parts) => Ok(parts),
            None => Err(PeshError::Evaluator(EvaluatorError::SplitError))?,
        }
    }

    pub fn eval(&self, command: &[String]) -> PeshResult<Command> {
        assert!(!command.is_empty());

        Command::try_from(command).map_err(PeshError::from)
    }

    pub fn eval_composite(&self, parts: &[String]) -> PeshResult<CompositeCommand> {
        #[derive(Default, Debug, Copy, Clone)]
        enum ParseState {
            #[default]
            Command,
            RedirStdout,
            RedirStderr,
        }

        let mut pstate = ParseState::default();
        let mut commands: Vec<_> = Vec::new();
        let mut stdout_path = None;
        let mut stderr_path = None;
        let mut current_command = Vec::new();

        macro_rules! submit_command {
            () => {
                if !current_command.is_empty() {
                    commands.push(self.eval(&current_command)?);
                    current_command.clear();
                }
            };
        }

        dbg!(&parts);
        for subpart in parts {
            dbg!(pstate);
            dbg!(&subpart);
            match pstate {
                ParseState::Command => {
                    if subpart == "1>" {
                        submit_command!();
                        pstate = ParseState::RedirStdout
                    } else if subpart == "2>" {
                        submit_command!();
                        pstate = ParseState::RedirStderr
                    } else if subpart == ";" {
                        submit_command!();
                        continue;
                    } else {
                        current_command.push(subpart.to_owned());
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

        let cc = CompositeCommand::new(&commands)
            .stdout_to(stdout_path)
            .stderr_to(stderr_path);
        dbg!(&cc);
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
