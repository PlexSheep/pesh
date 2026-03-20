pub mod command;
pub mod parser;
use parser::*;

use std::{
    fs,
    io::{self},
    path::PathBuf,
};

use nix::unistd::AccessFlags;

use crate::{
    error::{EvaluatorError, PeshError, PeshResult},
    eval::command::{CommandTask, composite::Command},
};

#[inline]
pub fn eval_raw(command_raw: &str) -> PeshResult<Command> {
    let mut normalized: String = String::new();

    let mut last_ch = '_';
    let mut chars = command_raw.chars();
    while let Some(ch) = chars.next() {
        if ch == '>' && last_ch != '1' && last_ch != '2' {
            if let Some(nch) = chars.next() {
                if nch == '>' {
                    normalized.push_str(" >> ");
                } else {
                    normalized.push_str(" > ");
                    normalized.push(nch);
                }
            } else {
                normalized.push_str(" > ");
            }
        } else if ch.is_numeric() || ch == '&' {
            if let Some(nch) = chars.next() {
                if nch == '>' {
                    if let Some(nnch) = chars.next() {
                        if nnch == '>' {
                            normalized.push_str(&format!(" {ch}>> "));
                        } else {
                            normalized.push_str(&format!(" {ch}> "));
                            normalized.push(nnch);
                        }
                    } else {
                        normalized.push_str(" > ");
                    }
                } else {
                    normalized.push(ch);
                    normalized.push(nch);
                }
            } else {
                normalized.push(ch);
            }
            continue;
        } else {
            normalized.push(ch);
        }
        last_ch = ch;
    }

    tracing::debug!("Normalized: r#\"{normalized}\"#");
    eval_command(&split(&normalized)?)
}

pub fn eval_task(command: &[String]) -> PeshResult<CommandTask> {
    assert!(!command.is_empty());

    CommandTask::try_from(command).map_err(PeshError::from)
}

pub fn eval_command(parts: &[String]) -> PeshResult<Command> {
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
    let mut stdout_append = false;
    let mut stderr_append = false;
    let mut argv = Vec::new();

    // TODO: this manual parsing sucks. Find a way to do this better
    for part in parts {
        match pstate {
            ParseState::Command => {
                if part == "1>" || part == ">" {
                    pstate = ParseState::RedirStdout
                } else if part == "1>>" || part == ">>" {
                    pstate = ParseState::RedirStdout;
                    stdout_append = true;
                } else if part == "2>" {
                    pstate = ParseState::RedirStderr
                } else if part == "2>>" {
                    pstate = ParseState::RedirStderr;
                    stderr_append = true;
                } else if part.chars().next().is_some_and(|c| c.is_numeric())
                    && part.chars().nth(1).is_some_and(|c| c == '>')
                {
                    todo!("only 1> and 2> are currently supported")
                } else if part == "&>" {
                    todo!("&> like redirections are not implemented")
                } else if part == ";" || part == "&&" || part == "||" {
                    todo!("multiple commands with ';' , '||' or '&&' are not implemented")
                } else {
                    argv.push(part.to_owned());
                }
            }
            ParseState::RedirStdout => {
                stdout_path = Some((part).into());
                pstate = ParseState::Command;
            }
            ParseState::RedirStderr => {
                stderr_path = Some((part).into());
                pstate = ParseState::Command;
            }
        }
    }
    let ct = eval_task(&argv)?;

    let cc = Command::new(ct)
        .with_stdout_to(stdout_path)
        .with_stderr_to(stderr_path)
        .with_stdout_append(stdout_append)
        .with_stderr_append(stderr_append);
    Ok(cc)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eval_split() {
        assert_eq!(split("hello world").unwrap(), ["hello", "world"]);
        assert_eq!(
            split("hello world 19193139-asfjkagsjiju==??").unwrap(),
            ["hello", "world", "19193139-asfjkagsjiju==??"]
        );
        assert_eq!(
            split("hello \"world of love\"").unwrap(),
            ["hello", "world of love"]
        );
        assert_eq!(
            split("hello \"world \\\"of love\"").unwrap(),
            ["hello", "world \"of love"]
        );
        assert_eq!(
            split("hello \"world's boom \\\"of love\"").unwrap(),
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
