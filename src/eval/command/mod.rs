pub mod builtins;
pub(crate) mod composite;
pub use composite::Command;

use std::{fmt::Display, path::PathBuf, str::FromStr};

use strum::IntoEnumIterator;
use strum_macros::{Display, EnumCount, EnumIter, EnumString};

use crate::error::EvaluatorError;

#[derive(Debug, Clone, Hash)]
pub enum CommandTask {
    Builtin(BuiltinCommand),
    Extern { argv: Vec<String> },
}

#[allow(nonstandard_style)] // these are literally the builtin commands
#[derive(Debug, Clone, Hash, Display, EnumCount, EnumIter, EnumString)]
pub enum BuiltinCommand {
    exit,
    pwd,
    cd(Option<std::path::PathBuf>),
    echo(Vec<String>),
    #[strum(serialize = "type")]
    r#type(String),
}

impl CommandTask {
    pub fn is_builtin(command: &[String]) -> bool {
        let mut is_builtin = false;
        for builtin in BuiltinCommand::iter() {
            is_builtin |= builtin.to_string() == command[0];
        }
        is_builtin
    }

    pub fn extern_argv(argv: Vec<String>) -> Self {
        Self::Extern { argv }
    }
}

fn guarantee_args(
    command: &[String],
    allowed: impl std::ops::RangeBounds<u8>,
) -> Result<(), EvaluatorError> {
    if !allowed.contains(&(command.len() as u8 - 1)) {
        return Err(EvaluatorError::WrongNumberOfArguments(
            command.len() as u8 - 1,
        ));
    }
    Ok(())
}

impl TryFrom<&[String]> for CommandTask {
    type Error = EvaluatorError;

    fn try_from(command: &[String]) -> Result<Self, Self::Error> {
        match BuiltinCommand::from_str(&command[0]) {
            Err(_) => (),
            Ok(mut builtin_command) => {
                match &builtin_command {
                    BuiltinCommand::r#type(_) => {
                        guarantee_args(command, 1..=1)?;
                        assert!(!command[1].is_empty());
                        builtin_command = BuiltinCommand::r#type(command[1].to_string())
                    }
                    BuiltinCommand::cd(_) => {
                        guarantee_args(command, ..=1)?;
                        builtin_command =
                            BuiltinCommand::cd(if let Some(path_raw) = command.get(1) {
                                let path = PathBuf::from_str(path_raw).expect("Infalliable");
                                Some(path)
                            } else {
                                None
                            })
                    }
                    BuiltinCommand::echo(_) => {
                        guarantee_args(command, 1..)?;
                        builtin_command = BuiltinCommand::echo(command[1..].to_vec())
                    }
                    _ => (),
                }
                return Ok(Self::Builtin(builtin_command));
            }
        }
        Ok(Self::extern_argv(command.to_vec()))
    }
}

impl Display for CommandTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Builtin(bi) => bi.to_string(),
                Self::Extern { argv, .. } => argv[0].to_string(),
            }
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn command_range_stupidity() {
        assert!((..=1).contains(&0));
        assert!((..=1).contains(&1));
        assert!((1..=1).contains(&1));
        assert!((1..).contains(&1));
        assert!((1..).contains(&2));
        guarantee_args(&["aa".to_string(), "bb".to_string()], 1..).unwrap();
    }

    #[test]
    fn command_builtin_checker() {
        assert!(CommandTask::is_builtin(&["cd".to_string()]));
        assert!(CommandTask::is_builtin(&[
            "cd".to_string(),
            "foo".to_string()
        ]));
        assert!(CommandTask::is_builtin(&[
            "type".to_string(),
            "bar".to_string()
        ]));
        assert!(CommandTask::is_builtin(&[
            "type".to_string(),
            "type".to_string()
        ]));
        assert!(!CommandTask::is_builtin(&["gabbakhjdksjfda".to_string()]));
    }
}
