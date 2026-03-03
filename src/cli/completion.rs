use dialoguer::Completion;

pub struct PeshCompletion {
    options: Vec<String>,
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
