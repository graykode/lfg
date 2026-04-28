use std::ffi::OsString;
use std::path::{Path, PathBuf};
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathCommandLocator {
    path_entries: Vec<PathBuf>,
    skip_paths: Vec<PathBuf>,
}

impl PathCommandLocator {
    pub fn from_env(skip_paths: impl IntoIterator<Item = PathBuf>) -> Self {
        Self::new(
            std::env::var_os("PATH").unwrap_or_default(),
            skip_paths.into_iter().collect(),
        )
    }

    pub fn new(path: OsString, skip_paths: Vec<PathBuf>) -> Self {
        Self {
            path_entries: std::env::split_paths(&path).collect(),
            skip_paths,
        }
    }

    pub fn resolve(&self, program: &str) -> Option<PathBuf> {
        let program_path = Path::new(program);
        if program_path.components().count() > 1 {
            return self
                .is_runnable_candidate(program_path)
                .then(|| program_path.to_path_buf());
        }

        self.path_entries
            .iter()
            .map(|path| path.join(program))
            .find(|candidate| self.is_runnable_candidate(candidate))
    }

    fn is_runnable_candidate(&self, candidate: &Path) -> bool {
        candidate.is_file() && !self.is_skipped(candidate)
    }

    fn is_skipped(&self, candidate: &Path) -> bool {
        self.skip_paths
            .iter()
            .any(|skip_path| same_file_or_path(candidate, skip_path))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessCommandExecutor {
    locator: PathCommandLocator,
    stream_stdio: bool,
}

impl ProcessCommandExecutor {
    pub fn for_invocation(program_path: impl Into<PathBuf>) -> Self {
        let mut skip_paths = vec![program_path.into()];
        if let Ok(current_executable) = std::env::current_exe() {
            skip_paths.push(current_executable);
        }

        Self {
            locator: PathCommandLocator::from_env(skip_paths),
            stream_stdio: true,
        }
    }
}

fn same_file_or_path(left: &Path, right: &Path) -> bool {
    if left == right {
        return true;
    }

    match (left.canonicalize(), right.canonicalize()) {
        (Ok(left), Ok(right)) => left == right,
        _ => false,
    }
}

impl CommandExecutor for ProcessCommandExecutor {
    fn execute(&self, command: &RealCommand) -> Result<CommandOutput, CommandExecutionError> {
        let program = self
            .locator
            .resolve(&command.program)
            .ok_or_else(|| CommandExecutionError::Unavailable(command.program.clone()))?;

        if self.stream_stdio {
            let status = Command::new(program)
                .args(&command.args)
                .status()
                .map_err(|error| {
                    if error.kind() == std::io::ErrorKind::NotFound {
                        CommandExecutionError::Unavailable(error.to_string())
                    } else {
                        CommandExecutionError::Failed(error.to_string())
                    }
                })?;

            return Ok(CommandOutput {
                exit_code: status.code().unwrap_or(1),
                stdout: String::new(),
                stderr: String::new(),
            });
        }

        let output = Command::new(program)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs::symlink;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_test_dir(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}-{nanos}"))
    }

    #[test]
    fn path_locator_skips_configured_paths_and_uses_next_matching_program() {
        let temp_dir = temp_test_dir("packvet-command-locator");
        let skipped_dir = temp_dir.join("skipped");
        let real_dir = temp_dir.join("real");
        fs::create_dir_all(&skipped_dir).expect("create skipped dir");
        fs::create_dir_all(&real_dir).expect("create real dir");

        let packvet_path = temp_dir.join("packvet");
        let skipped_path = skipped_dir.join("npm");
        let real_path = real_dir.join("npm");
        fs::write(&packvet_path, "").expect("write packvet placeholder");
        symlink(&packvet_path, &skipped_path).expect("create skipped symlink");
        fs::write(&real_path, "").expect("write real npm placeholder");

        let locator = PathCommandLocator::new(
            std::env::join_paths([&skipped_dir, &real_dir]).expect("join path"),
            vec![skipped_path],
        );

        assert_eq!(locator.resolve("npm"), Some(real_path));

        fs::remove_dir_all(temp_dir).expect("remove temp dir");
    }
}
