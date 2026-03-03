use std::io::{self, Write};
use std::process::ExitCode;

use chrono::{NaiveTime, TimeDelta};
use clap::{Parser, Subcommand};

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
    /// Type of time operation to perform
    #[arg(short, long, value_name = "COMMMAND")]
    command: Option<String>,
}

pub struct Cli {
    args: CliArgs,
    interactive: bool,
    eval: Evaluator,
}
impl Cli {
    fn interactive(&self) -> PeshResult<ExitCode> {
        print!("$ ");
        io::stdout().flush()?;
        Ok(ExitCode::SUCCESS)
    }
}

pub fn cli(args: &[String]) -> ExitCode {
    let cli: Cli = CliArgs::parse_from(args).into();

    let res = if cli.interactive {
        cli.interactive()
    } else if let Some(command) = cli.args.command {
        cli.eval.eval_raw(command)
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
        let c = Cli {
            interactive: args.command.is_none(),
            eval: Evaluator::default(),
            args,
        };

        #[cfg(debug_assertions)]
        {
            assert_eq!(c.interactive, c.args.command.is_none());
        }

        c
    }
}
