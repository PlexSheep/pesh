pub mod completion;
pub mod theme;

use std::process::ExitCode;
use std::{fs, io};
use std::{io::Write, path::Path};

use clap::Parser;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{BasicHistory, Input};

use crate::cli::completion::PeshCompletion;
use crate::cli::theme::{Theme, posix::PosixTheme};
use crate::error::PeshResult;
use crate::error::{EvaluatorError, PeshError};
use crate::eval::command::builtins::{
    builtin_command_cd, builtin_command_echo, builtin_command_pwd, builtin_command_type,
};
use crate::eval::command::{BuiltinCommand, Command, CommandTask};
use crate::eval::{eval_raw, locate_executable};
use crate::out_stream::Redirects;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about,
    long_about,
    help_template = r#"{about-section}
{usage-heading} {usage}
{all-args}{tab}

{name}: {version}
Author: {author-with-newline}
"#
)]
pub struct CliArgs {
    /// Execute a given command and exit, non interactive
    #[arg(short, long, value_name = "COMMMAND")]
    command: Option<String>,

    /// POSIX compliant behavior
    #[arg(short, long)]
    posix: bool,

    /// Print debug information
    #[arg(short, long)]
    debug: bool,
}

pub struct Cli {
    args: CliArgs,
    interactive: bool,

    input_theme: Theme,
    input_completion: PeshCompletion,
    input_history: BasicHistory,
}

impl Cli {
    pub fn interactive(&mut self) -> PeshResult<ExitCode> {
        let mut input;
        let mut command;
        loop {
            input = self.input()?;
            command = eval_raw(&input)?;
            if matches!(command.task(), CommandTask::Builtin(BuiltinCommand::exit)) {
                break;
            }
            self.execute_command(command)?;
        }
        Ok(ExitCode::SUCCESS)
    }

    pub fn input(&mut self) -> PeshResult<String> {
        Input::<String>::with_theme(self.input_theme.downcast())
            .with_prompt("$")
            .history_with(&mut self.input_history)
            .completion_with(&self.input_completion)
            .interact_text()
            .map_err(PeshError::from)
    }

    pub fn open_path_for_output(path: &Path, trunc: bool) -> PeshResult<fs::File> {
        let file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(trunc)
            .append(!trunc)
            .open(path)?;
        Ok(file)
    }

    pub fn execute_command(&self, command: Command) -> PeshResult<ExitCode> {
        let redirs = Redirects {
            stdin: io::stdin(),
            stdout: if let Some(path) = command.stdout_to() {
                Self::open_path_for_output(path, !command.stdout_append())?.into()
            } else {
                io::stdout().into()
            },
            stderr: if let Some(path) = command.stderr_to() {
                Self::open_path_for_output(path, !command.stderr_append())?.into()
            } else {
                io::stderr().into()
            },
        };

        let ret = self.execute_command_task(command.task(), redirs)?;

        Ok(ret)
    }

    pub fn execute_command_task(
        &self,
        command: &CommandTask,
        mut redirs: Redirects,
    ) -> PeshResult<ExitCode> {
        let ret = match &command {
            CommandTask::Builtin(bi) => match &bi {
                BuiltinCommand::exit => unreachable!(),
                BuiltinCommand::r#type(arg) => builtin_command_type(&mut redirs, arg),
                BuiltinCommand::pwd => builtin_command_pwd(&mut redirs),
                BuiltinCommand::echo(args) => builtin_command_echo(&mut redirs, args),
                BuiltinCommand::cd(arg) => builtin_command_cd(&mut redirs, arg.as_ref()),
            },
            CommandTask::Extern { argv, .. } => {
                let path_env = std::env::var("PATH").unwrap_or("".to_string());

                match locate_executable(&path_env, &argv[0])? {
                    Some(_path) => {
                        let mut child = std::process::Command::new(&argv[0])
                            .args(&argv[1..])
                            .stdout(redirs.stdout)
                            .stderr(redirs.stderr)
                            .spawn()?;
                        let res = child.wait()?;
                        Ok(if res.success() {
                            ExitCode::SUCCESS
                        } else {
                            ExitCode::FAILURE
                        })
                    }
                    None => Err(EvaluatorError::CommandNotFound.into()),
                }
            }
        };
        match ret {
            Ok(ex) => Ok(ex),
            Err(err) => {
                eprintln!("{}: {err}", &command);
                Ok(ExitCode::FAILURE)
            }
        }
    }
}

pub fn bell() {
    print!("\x07");
    io::stdout().flush().expect("could not write to stdout");
}

fn cli_inner(args: &[String]) -> PeshResult<ExitCode> {
    let mut cli: Cli = CliArgs::parse_from(args).into();

    if cli.interactive {
        cli.interactive()
    } else if let Some(command) = &cli.args.command {
        cli.execute_command(eval_raw(command)?)
    } else {
        unreachable!()
    }
}

pub fn cli(args: &[String]) -> ExitCode {
    match cli_inner(args) {
        Err(err) => {
            eprintln!("{err}");
            ExitCode::FAILURE
        }
        Ok(ec) => ec,
    }
}

impl From<CliArgs> for Cli {
    fn from(args: CliArgs) -> Self {
        let input_completion = PeshCompletion::default();
        let input_theme = if args.posix {
            Theme::Posix(PosixTheme)
        } else {
            Theme::Fancy(ColorfulTheme::default())
        };
        let input_history = BasicHistory::new().no_duplicates(true);

        let c = Cli {
            interactive: args.command.is_none(),
            input_completion,
            input_theme,
            input_history,
            args,
        };

        #[cfg(debug_assertions)]
        {
            assert_eq!(c.interactive, c.args.command.is_none());
        }

        if c.args.debug {
            use tracing_subscriber::prelude::*;
            tracing_subscriber::registry()
                .with(tracing_subscriber::fmt::layer().without_time())
                .init();
        }

        c
    }
}
