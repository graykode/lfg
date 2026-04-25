use std::io::Write;
use std::process::{Command, Stdio};

use crate::providers::{ProviderError, ReviewPrompt, ReviewProvider};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandReviewProvider {
    id: &'static str,
    program: String,
    args: Vec<String>,
}

impl CommandReviewProvider {
    pub fn new<I, S>(id: &'static str, program: impl Into<String>, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            id,
            program: program.into(),
            args: args.into_iter().map(Into::into).collect(),
        }
    }
}

impl ReviewProvider for CommandReviewProvider {
    fn id(&self) -> &'static str {
        self.id
    }

    fn review(&self, prompt: &ReviewPrompt) -> Result<String, ProviderError> {
        let mut child = Command::new(&self.program)
            .args(&self.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| {
                if error.kind() == std::io::ErrorKind::NotFound {
                    ProviderError::Unavailable(error.to_string())
                } else {
                    ProviderError::Failure(error.to_string())
                }
            })?;

        let Some(mut stdin) = child.stdin.take() else {
            return Err(ProviderError::Failure(
                "provider stdin is unavailable".to_owned(),
            ));
        };

        stdin
            .write_all(prompt.text.as_bytes())
            .map_err(|error| ProviderError::Failure(error.to_string()))?;
        drop(stdin);

        let output = child
            .wait_with_output()
            .map_err(|error| ProviderError::Failure(error.to_string()))?;
        if !output.status.success() {
            return Err(ProviderError::Failure(
                String::from_utf8_lossy(&output.stderr).into_owned(),
            ));
        }

        String::from_utf8(output.stdout).map_err(|error| ProviderError::Failure(error.to_string()))
    }
}
