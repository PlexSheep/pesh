use std::{collections::HashSet, sync::RwLock};

use dialoguer::Completion;
use strum::IntoEnumIterator;

use crate::{
    cli::{bell, binaries_in_path, path_from_env},
    eval::command::BuiltinCommand,
};

#[derive(Debug)]
pub struct PeshCompletion {
    // unfortunately, due to constraints of the dialoguer completion api, we require interior
    // mutuability for things like scrolling through completion options with multiple <TAB> inputs.
    inner: RwLock<Inner>,
}

#[derive(Debug, Clone)]
struct Inner {
    last_input: Option<String>,
    last_completion: Option<String>,
    last_completion_shifts: usize,
}

impl Inner {}

impl PeshCompletion {
    pub fn new() -> Self {
        Self {
            inner: Inner {
                last_input: None,
                last_completion: None,
                last_completion_shifts: 0,
            }
            .into(),
        }
    }

    fn inner(&self) -> std::sync::RwLockReadGuard<'_, Inner> {
        self.inner.read().unwrap()
    }

    fn inner_mut(&self) -> std::sync::RwLockWriteGuard<'_, Inner> {
        self.inner.write().unwrap()
    }

    fn last_completion_shifts(&self) -> usize {
        self.inner().last_completion_shifts
    }

    fn set_last_completion_shifts(&self, last_completion_shifts: usize) {
        self.inner_mut().last_completion_shifts = last_completion_shifts;
    }

    fn last_completion(&self) -> Option<String> {
        self.inner().last_completion.clone()
    }

    fn set_last_completion(&self, last_completion: Option<String>) {
        self.inner_mut().last_completion = last_completion;
    }

    fn last_input(&self) -> Option<String> {
        self.inner().last_input.clone()
    }

    fn set_last_input(&self, last_input: Option<String>) {
        self.inner_mut().last_input = last_input;
    }

    fn find_matches_for(&self, input: &str) -> HashSet<String> {
        let mut options: HashSet<String> = BuiltinCommand::iter().map(|c| c.to_string()).collect();
        options.extend(
            binaries_in_path(&path_from_env())
                .unwrap_or(vec![])
                .into_iter()
                .filter(|b| b.file_name().is_some())
                .map(|b| b.file_name().unwrap().to_string_lossy().to_string()),
        );

        options
            .into_iter()
            .filter(|s| s.starts_with(input))
            .collect()
    }
}

impl Default for PeshCompletion {
    fn default() -> Self {
        Self::new()
    }
}

impl Completion for PeshCompletion {
    /// Simple completion implementation based on substring
    fn get(&self, input: &str) -> Option<String> {
        let mut followup = false;

        tracing::debug!("input: {input}");
        tracing::debug!("last_input: {:?}", self.last_input());

        if let Some(s) = &self.last_completion()
            && !s.starts_with(input)
        {
            self.set_last_completion(None);
            self.set_last_completion_shifts(0);

            followup = true;
        }

        tracing::debug!("followup: {followup}");
        tracing::debug!(
            "prev: {:?} ({})",
            self.last_completion(),
            self.last_completion_shifts()
        );

        let matches = self.find_matches_for(input);

        tracing::debug!("matches: {matches:?}");

        let res = matches.into_iter().nth(self.last_completion_shifts());

        self.set_last_completion(res.clone());
        if !followup {
            self.set_last_input(Some(input.to_string()));
        }

        if res.is_none() {
            bell();
        }
        res
    }
}
