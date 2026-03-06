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
    if CommandTask::is_builtin(&[arg.to_string()]) {
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
        write!(r.stdout, "{arg}")?;
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
