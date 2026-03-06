pub mod completion;
pub mod theme;

use std::io;
use std::path::Path;
use std::process::ExitCode;

use clap::Parser;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{BasicHistory, Input};

use crate::cli::completion::PeshCompletion;
use crate::cli::theme::{Theme, posix::PosixTheme};
use crate::error::{EvaluatorError, PeshError};
use crate::eval::command::builtins::{
    builtin_command_cd, builtin_command_echo, builtin_command_pwd, builtin_command_type,
};
use crate::eval::command::{BuiltinCommand, Command, CompositeCommand, Redirects};
use crate::eval::locate_executable;
use crate::{error::PeshResult, eval::Evaluator};

/// zeitr - Time calculation utility
///
/// A tool for time calculations,
/// for tracking work hours, project time spans,
/// and performing time arithmetic operations.
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
}

pub struct Cli {
    args: CliArgs,
    interactive: bool,
    eval: Evaluator,

    input_theme: Theme,
    input_completion: PeshCompletion,
    input_history: BasicHistory,
}

impl Cli {
    pub fn interactive(&mut self) -> PeshResult<ExitCode> {
        let mut input;
        let mut comp_command;
        loop {
            input = self.input()?;
            comp_command = self.eval.eval_raw(&input)?;
            if comp_command.commands_len() == 1
                && matches!(
                    comp_command.commands()[0],
                    Command::Builtin(BuiltinCommand::exit)
                )
            {
                break;
            }
            self.execute_comp_command(comp_command)?;
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

    pub fn execute_comp_command(&self, comp_command: CompositeCommand) -> PeshResult<ExitCode> {
        let mut ret = ExitCode::SUCCESS;
        let mut redirs = Redirects {
            stdin: io::stdin(),
            stdout: io::stdout(),
            stderr: io::stderr(),
        };

        for command in comp_command.commands() {
            ret = self.execute_command(command, &mut redirs)?;
            if ret != ExitCode::SUCCESS {
                return Ok(ret);
            }
        }
        Ok(ret)
    }

    pub fn execute_command(
        &self,
        command: &Command,
        redirs: &mut Redirects,
    ) -> PeshResult<ExitCode> {
        let ret = match &command {
            Command::Builtin(bi) => match &bi {
                BuiltinCommand::exit => unreachable!(),
                BuiltinCommand::r#type(arg) => builtin_command_type(redirs, arg),
                BuiltinCommand::pwd => builtin_command_pwd(redirs),
                BuiltinCommand::echo(args) => builtin_command_echo(redirs, args),
                BuiltinCommand::cd(arg) => builtin_command_cd(redirs, arg.as_ref()),
            },
            Command::Extern { argv: ei, .. } => {
                let path_env = std::env::var("PATH").unwrap_or("".to_string());

                match locate_executable(&path_env, &ei[0])? {
                    Some(_path) => {
                        let mut child =
                            std::process::Command::new(&ei[0]).args(&ei[1..]).spawn()?;
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

fn cli_inner(args: &[String]) -> PeshResult<ExitCode> {
    let mut cli: Cli = CliArgs::parse_from(args).into();

    if cli.interactive {
        cli.interactive()
    } else if let Some(command) = &cli.args.command {
        cli.execute_comp_command(cli.eval.eval_raw(command)?)
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
            eval: Evaluator::default(),
            input_completion,
            input_theme,
            input_history,
            args,
        };

        #[cfg(debug_assertions)]
        {
            assert_eq!(c.interactive, c.args.command.is_none());
        }

        c
    }
}
