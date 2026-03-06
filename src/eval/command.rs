use std::{fmt::Display, path::PathBuf, str::FromStr};

use strum::IntoEnumIterator;
use strum_macros::{Display, EnumCount, EnumIter, EnumString};

use crate::error::EvaluatorError;

#[derive(Debug, Clone, Hash)]
pub struct CompositeCommand {
    commands: Vec<Command>,
    stdout_to: Option<PathBuf>,
    stderr_to: Option<PathBuf>,
}

impl CompositeCommand {
    pub fn new(commands: &[Command]) -> Self {
        Self {
            commands: commands.to_vec(),
            stdout_to: None,
            stderr_to: None,
        }
    }

    pub fn with_stdout_to(mut self, path: Option<PathBuf>) -> Self {
        self.stdout_to = path;
        self
    }

    pub fn with_stderr_to(mut self, path: Option<PathBuf>) -> Self {
        self.stderr_to = path;
        self
    }

    #[inline]
    pub fn commands(&self) -> &[Command] {
        &self.commands
    }

    #[inline]
    pub fn commands_len(&self) -> usize {
        self.commands().len()
    }

    pub fn stderr_to(&self) -> Option<&PathBuf> {
        self.stderr_to.as_ref()
    }

    pub fn stdout_to(&self) -> Option<&PathBuf> {
        self.stdout_to.as_ref()
    }
}

#[derive(Debug, Clone, Hash)]
pub enum Command {
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

impl Command {
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

impl TryFrom<&[String]> for Command {
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

impl Display for Command {
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

pub mod builtins {
    use std::{
        env,
        io::{ErrorKind, Write},
        process::ExitCode,
    };

    use crate::{
        error::PeshResult,
        eval::{get_home, locate_executable},
        out_stream::Redirects,
    };

    use super::*;

    pub fn builtin_command_type(r: &mut Redirects, arg: &str) -> PeshResult<ExitCode> {
        if Command::is_builtin(&[arg.to_string()]) {
            writeln!(r.stdout, "{} is a shell builtin", arg)?;
            Ok(ExitCode::SUCCESS)
        } else {
            let path_env = std::env::var("PATH").unwrap_or("".to_string());
            match locate_executable(&path_env, arg)? {
                Some(path) => {
                    writeln!(r.stdout, "{} is {}", arg, path.to_string_lossy())?;
                    Ok(ExitCode::SUCCESS)
                }
                None => {
                    // we handle this somewhat strange edgecase here, without the error system
                    writeln!(r.stderr, "{arg} not found")?;
                    Ok(ExitCode::FAILURE)
                }
            }
        }
    }

    pub fn builtin_command_pwd(r: &mut Redirects) -> PeshResult<ExitCode> {
        writeln!(
            r.stdout,
            "{}",
            env::current_dir()
                .expect("no current working directory")
                .to_string_lossy()
        )?;
        Ok(ExitCode::SUCCESS)
    }

    pub fn builtin_command_echo(r: &mut Redirects, args: &[String]) -> PeshResult<ExitCode> {
        // TODO: adding command line args for the builtin echo would be neat
        for (i, arg) in args.iter().enumerate() {
            if i != 0 {
                write!(r.stdout, " ")?;
            }
            print!("{arg}");
            if i + 1 == args.len() {
                writeln!(r.stdout)?;
            }
        }
        Ok(ExitCode::SUCCESS)
    }

    pub fn builtin_command_cd(_r: &mut Redirects, arg: Option<&PathBuf>) -> PeshResult<ExitCode> {
        // TODO: implement going back multiple directories with multiple dots
        let path = match arg {
            Some(a) if a == &Into::<PathBuf>::into("~") => get_home(),
            None => get_home(),
            Some(a) => a.to_owned(),
        };

        if let Err(err) = std::env::set_current_dir(&path) {
            if err.kind() == ErrorKind::NotFound {
                Err(EvaluatorError::FileOrDirNotFound(path))?
            } else {
                Err(err)?
            }
        }

        Ok(ExitCode::SUCCESS)
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
        assert!(Command::is_builtin(&["cd".to_string()]));
        assert!(Command::is_builtin(&["cd".to_string(), "foo".to_string()]));
        assert!(Command::is_builtin(&[
            "type".to_string(),
            "bar".to_string()
        ]));
        assert!(Command::is_builtin(&[
            "type".to_string(),
            "type".to_string()
        ]));
        assert!(!Command::is_builtin(&["gabbakhjdksjfda".to_string()]));
    }
}
