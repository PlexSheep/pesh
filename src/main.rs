use std::{env, process::ExitCode};

use pesh::error::PeshResult;

fn main() -> PeshResult<ExitCode> {
    let args: Vec<String> = env::args().collect();
    pesh::cli::cli(&args)
}
