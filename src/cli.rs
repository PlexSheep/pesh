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

#[allow(clippy::large_enum_variant)]
pub enum Theme {
    Fancy(ColorfulTheme),
    Posix(SimpleTheme),
}

pub struct PeshCompletion {
    options: Vec<String>,
}

impl Theme {
    fn downcast(&self) -> &dyn dialoguer::theme::Theme {
        match self {
            Self::Fancy(t) => t,
            Self::Posix(t) => t,
        }
    }
}

impl Cli {
    pub fn interactive(&mut self) -> PeshResult<ExitCode> {
        let mut command;
        let mut ret;
        loop {
            command = self.input()?;
            ret = self.eval.eval_raw(&command);
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
        let input_theme = if args.posix {
            Theme::Posix(SimpleTheme)
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
