pub mod completion;
pub mod theme;

use std::env;
use std::process::ExitCode;

use clap::Parser;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{BasicHistory, Input};

use crate::cli::completion::PeshCompletion;
use crate::cli::theme::{Theme, posix::PosixTheme};
use crate::error::{EvaluatorError, PeshError};
use crate::eval::command::{BuiltinCommand, Command};
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
        let mut command;
        let mut ret;
        loop {
            input = self.input()?;
            command = self.eval.eval_raw(&input)?;
            if matches!(command, Command::Builtin(BuiltinCommand::exit)) {
                break;
            }
            ret = self.execute_command(command);
            if let Err(e) = ret {
                eprintln!("{e}")
            }
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

    pub fn execute_command(&self, command: Command) -> PeshResult<ExitCode> {
        match command {
            Command::Builtin(bi) => match &bi {
                BuiltinCommand::r#type(arg) => {
                    if Command::is_builtin(&[arg.to_string()]) {
                        println!("{} is a shell builtin", arg);
                        Ok(ExitCode::SUCCESS)
                    } else {
                        Err(PeshError::Evaluator(
                            arg.to_string(),
                            EvaluatorError::CommandNotFound,
                        ))
                    }
                }
                BuiltinCommand::pwd => {
                    println!(
                        "{}",
                        env::current_dir()
                            .expect("no current working directory")
                            .to_string_lossy()
                    );
                    Ok(ExitCode::SUCCESS)
                }
                BuiltinCommand::echo(args) => {
                    // TODO: adding command line args for the builtin echo would be neat
                    for (i, arg) in args.iter().enumerate() {
                        if i != 0 {
                            print!(" ");
                        }
                        print!("{arg}");
                        if i + 1 == args.len() {
                            println!()
                        }
                    }
                    Ok(ExitCode::SUCCESS)
                }
                BuiltinCommand::exit => unreachable!(),
                other => {
                    todo!("{other} is not yet implemented")
                }
            },
            Command::Extern(ei) => Err(PeshError::Evaluator(
                ei[0].to_string(),
                EvaluatorError::CommandNotFound,
            )),
        }
    }
}

fn cli_inner(args: &[String]) -> PeshResult<ExitCode> {
    let mut cli: Cli = CliArgs::parse_from(args).into();

    if cli.interactive {
        cli.interactive()
    } else if let Some(command) = &cli.args.command {
        cli.execute_command(cli.eval.eval_raw(command)?)
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
