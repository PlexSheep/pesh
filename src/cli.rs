use std::io::{self, Write};
use std::process::ExitCode;

use clap::Parser;
use dialoguer::theme::{ColorfulTheme, SimpleTheme};
use dialoguer::{BasicHistory, Completion, Input};

use crate::error::PeshError;
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
}

pub struct Cli {
    args: CliArgs,
    interactive: bool,
    eval: Evaluator,

    input_theme: ColorfulTheme,
    input_completion: PeshCompletion,
}

pub struct PeshCompletion {
    options: Vec<String>,
}

impl Cli {
    pub fn interactive(&self) -> PeshResult<ExitCode> {
        let mut command;
        loop {
            command = self.input()?;
            self.eval.eval_raw(&command);
        }
        Ok(ExitCode::SUCCESS)
    }

        Input::<String>::with_theme(&self.input_theme)
    pub fn input(&mut self) -> PeshResult<String> {
            .with_prompt("$")
            .history_with(&mut self.input_history)
            .completion_with(&self.input_completion)
            .interact_text()
            .map_err(PeshError::from)
    }
}

pub fn cli(args: &[String]) -> ExitCode {
    let mut cli: Cli = CliArgs::parse_from(args).into();

    let res = if cli.interactive {
        cli.interactive()
    } else if let Some(command) = cli.args.command {
        cli.eval.eval_raw(&command)
    } else {
        unreachable!()
    };

    match res {
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
        let input_theme = ColorfulTheme::default();

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

impl Default for PeshCompletion {
    fn default() -> Self {
        PeshCompletion {
            options: vec!["pwd".to_string(), "cd".to_string()],
        }
    }
}

impl Completion for PeshCompletion {
    /// Simple completion implementation based on substring
    fn get(&self, input: &str) -> Option<String> {
        let matches = self
            .options
            .iter()
            .filter(|option| option.starts_with(input))
            .collect::<Vec<_>>();

        if matches.len() == 1 {
            Some(matches[0].to_string())
        } else {
            None
        }
    }
}
