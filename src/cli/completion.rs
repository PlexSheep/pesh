use dialoguer::Completion;
use strum::IntoEnumIterator;

use crate::eval::command::BuiltinCommand;

#[derive(Default, Debug)]
pub struct PeshCompletion {}

impl Completion for PeshCompletion {
    /// Simple completion implementation based on substring
    fn get(&self, input: &str) -> Option<String> {
        let mut matches = BuiltinCommand::iter()
            .map(|v| format!("{v} "))
            .filter(|s| s.starts_with(input));

        matches.next()
    }
}
