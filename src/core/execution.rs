use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RealCommand {
    pub program: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandExecutionError {
    Unavailable(String),
    Failed(String),
}

pub trait CommandExecutor {
    fn execute(&self, command: &RealCommand) -> Result<CommandOutput, CommandExecutionError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProcessCommandExecutor;

impl CommandExecutor for ProcessCommandExecutor {
    fn execute(&self, command: &RealCommand) -> Result<CommandOutput, CommandExecutionError> {
        let output = Command::new(&command.program)
            .args(&command.args)
            .output()
            .map_err(|error| {
                if error.kind() == std::io::ErrorKind::NotFound {
                    CommandExecutionError::Unavailable(error.to_string())
                } else {
                    CommandExecutionError::Failed(error.to_string())
                }
            })?;

        Ok(CommandOutput {
            exit_code: output.status.code().unwrap_or(1),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
    }
}
