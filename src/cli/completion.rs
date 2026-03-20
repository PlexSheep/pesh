use dialoguer::Completion;
use strum::IntoEnumIterator;

use crate::{
    cli::{bell, binaries_in_path, path_from_env},
    eval::command::BuiltinCommand,
};

#[derive(Default, Debug)]
pub struct PeshCompletion {}

impl Completion for PeshCompletion {
    /// Simple completion implementation based on substring
    fn get(&self, input: &str) -> Option<String> {
        let mut options: Vec<String> = BuiltinCommand::iter().map(|c| c.to_string()).collect();
        options.extend(
            binaries_in_path(&path_from_env())
                .unwrap_or(vec![])
                .into_iter()
                .filter(|b| b.file_name().is_some())
                .map(|b| b.file_name().unwrap().to_string_lossy().to_string()),
        );

        let mut matches = options
            .into_iter()
            .map(|v| format!("{v} "))
            .filter(|s| s.starts_with(input));

        let res = matches.next();
        if res.is_none() {
            bell();
        }
        res
    }
}
