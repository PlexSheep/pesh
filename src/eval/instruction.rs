use std::str::FromStr;

use strum::IntoEnumIterator;
use strum_macros::{Display, EnumCount, EnumIter, EnumString};

use crate::error::EvaluatorError;

#[derive(Debug, Clone, Hash)]
pub enum Instruction {
    Builtin(BuiltinInstruction),
    Extern(Vec<String>),
}

#[allow(nonstandard_style)] // these are literally the builtin commands
#[derive(Debug, Clone, Hash, Display, EnumCount, EnumIter, EnumString)]
pub enum BuiltinInstruction {
    exit,
    pwd,
    cd(Option<std::path::PathBuf>),
}

impl TryFrom<&[String]> for Instruction {
    type Error = EvaluatorError;

    fn try_from(command: &[String]) -> Result<Self, Self::Error> {
        match BuiltinInstruction::from_str(&command[0]) {
            Err(_) => (),
            Ok(builtin_command) => {
                return Ok(Self::Builtin(builtin_command));
            }
        }
        Ok(Self::Extern(command.to_vec()))
    }
}
