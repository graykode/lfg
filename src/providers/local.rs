use std::env;
use std::io::{self, Read, Write};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::providers::{ProviderError, ReviewPrompt, ReviewProvider};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandReviewProvider {
    id: &'static str,
    program: String,
    args: Vec<String>,
    timeout: Duration,
}

impl CommandReviewProvider {
    const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);

    pub fn new<I, S>(id: &'static str, program: impl Into<String>, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self::with_timeout(id, program, args, Self::DEFAULT_TIMEOUT)
    }

    pub fn with_timeout<I, S>(
        id: &'static str,
        program: impl Into<String>,
        args: I,
        timeout: Duration,
    ) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            id,
            program: program.into(),
            args: args.into_iter().map(Into::into).collect(),
            timeout,
        }
    }
}

impl ReviewProvider for CommandReviewProvider {
    fn id(&self) -> &'static str {
        self.id
    }

    fn review(&self, prompt: &ReviewPrompt) -> Result<String, ProviderError> {
        if should_print_review_prompt() {
            write_review_prompt_to_stderr(prompt);
        }

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

        let status = wait_with_timeout(&mut child, self.timeout)?;
        let stdout = read_child_pipe(child.stdout.take())?;
        let stderr = read_child_pipe(child.stderr.take())?;

        if !status.success() {
            return Err(ProviderError::Failure(
                String::from_utf8_lossy(&stderr).into_owned(),
            ));
        }

        String::from_utf8(stdout).map_err(|error| ProviderError::Failure(error.to_string()))
    }
}

fn wait_with_timeout(child: &mut Child, timeout: Duration) -> Result<ExitStatus, ProviderError> {
    let started_at = Instant::now();

    loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|error| ProviderError::Failure(error.to_string()))?
        {
            return Ok(status);
        }

        if started_at.elapsed() >= timeout {
            let _ = child.kill();
            let _ = child.wait();
            return Err(ProviderError::Timeout);
        }

        thread::sleep(Duration::from_millis(10));
    }
}

fn read_child_pipe<R: Read>(pipe: Option<R>) -> Result<Vec<u8>, ProviderError> {
    let Some(mut pipe) = pipe else {
        return Ok(Vec::new());
    };
    let mut bytes = Vec::new();
    pipe.read_to_end(&mut bytes)
        .map_err(|error| ProviderError::Failure(error.to_string()))?;
    Ok(bytes)
}

fn should_print_review_prompt() -> bool {
    matches!(
        env::var("PACKVET_PRINT_REVIEW_PROMPT").ok().as_deref(),
        Some("1" | "true" | "yes")
    )
}

fn write_review_prompt_to_stderr(prompt: &ReviewPrompt) {
    let mut stderr = io::stderr().lock();
    let _ = writeln!(stderr, "----- packvet review prompt -----")
        .and_then(|_| write!(stderr, "{}", prompt.text))
        .and_then(|_| {
            if prompt.text.ends_with('\n') {
                Ok(())
            } else {
                writeln!(stderr)
            }
        })
        .and_then(|_| writeln!(stderr, "----- end packvet review prompt -----"));
}
