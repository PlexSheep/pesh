use std::path::PathBuf;

use crate::eval::command::CommandTask;

#[derive(Debug, Clone, Hash)]
pub struct Command {
    task: CommandTask,
    stdout_to: Option<PathBuf>,
    stderr_to: Option<PathBuf>,
    stdout_append: bool,
    stderr_append: bool,
}

impl Command {
    pub fn new(command: CommandTask) -> Self {
        Self {
            task: command,
            stdout_to: None,
            stderr_to: None,
            stdout_append: false,
            stderr_append: false,
        }
    }

    pub fn with_stdout_append(mut self, append: bool) -> Self {
        self.stdout_append = append;
        self
    }

    pub fn with_stderr_append(mut self, append: bool) -> Self {
        self.stderr_append = append;
        self
    }

    pub fn with_stdout_to(mut self, path: Option<PathBuf>) -> Self {
        self.stdout_to = path;
        self
    }

    pub fn with_stderr_to(mut self, path: Option<PathBuf>) -> Self {
        self.stderr_to = path;
        self
    }

    pub fn stderr_to(&self) -> Option<&PathBuf> {
        self.stderr_to.as_ref()
    }

    pub fn stdout_to(&self) -> Option<&PathBuf> {
        self.stdout_to.as_ref()
    }

    pub fn task(&self) -> &CommandTask {
        &self.task
    }
}

impl From<CommandTask> for Command {
    fn from(value: CommandTask) -> Self {
        Self::new(value)
    }
}
