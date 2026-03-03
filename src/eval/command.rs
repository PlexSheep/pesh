use std::{path::PathBuf, str::FromStr};

use strum_macros::{Display, EnumCount, EnumIter, EnumString};

use crate::error::EvaluatorError;

#[derive(Debug, Clone, Hash)]
pub enum Command {
    Builtin(BuiltinCommand),
    Extern(Vec<String>),
}

#[allow(nonstandard_style)] // these are literally the builtin commands
#[derive(Debug, Clone, Hash, Display, EnumCount, EnumIter, EnumString)]
pub enum BuiltinCommand {
    exit,
    pwd,
    cd(Option<std::path::PathBuf>),
    echo(Vec<String>),
}

impl TryFrom<&[String]> for Command {
    type Error = EvaluatorError;

    fn try_from(command: &[String]) -> Result<Self, Self::Error> {
        match BuiltinCommand::from_str(&command[0]) {
            Err(_) => (),
            Ok(mut builtin_command) => {
                match &builtin_command {
                    BuiltinCommand::cd(_) => {
                        builtin_command =
                            BuiltinCommand::cd(if let Some(path_raw) = command.get(1) {
                                let path = PathBuf::from_str(path_raw).expect("Infalliable");
                                Some(path)
                            } else {
                                None
                            })
                    }
                    BuiltinCommand::echo(_) => {
                        builtin_command = BuiltinCommand::echo(command[1..].to_vec())
                    }
                    _ => (),
                }
                return Ok(Self::Builtin(builtin_command));
            }
        }
        Ok(Self::Extern(command.to_vec()))
    }
}
