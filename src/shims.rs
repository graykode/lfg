use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShimCommand {
    Install { manager_id: String, dir: PathBuf },
    Uninstall { manager_id: String, dir: PathBuf },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShimCommandError {
    MissingAction,
    MissingDir,
    MissingManager,
    UnsupportedAction(String),
    UnknownArgument(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShimSetupError {
    ExistingPath(PathBuf),
    NotPackvetShim(PathBuf),
    Io(String),
}

pub fn parse_shim_command(args: &[String]) -> Result<ShimCommand, ShimCommandError> {
    let Some(action) = args.first() else {
        return Err(ShimCommandError::MissingAction);
    };

    let mut dir = None;
    let mut manager_id = None;
    let mut index = 1;

    while index < args.len() {
        match args[index].as_str() {
            "--dir" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err(ShimCommandError::MissingDir);
                };
                dir = Some(PathBuf::from(value));
            }
            argument if argument.starts_with('-') => {
                return Err(ShimCommandError::UnknownArgument(argument.to_owned()));
            }
            argument => {
                manager_id = Some(argument.to_owned());
            }
        }

        index += 1;
    }

    let Some(dir) = dir else {
        return Err(ShimCommandError::MissingDir);
    };
    let Some(manager_id) = manager_id else {
        return Err(ShimCommandError::MissingManager);
    };

    match action.as_str() {
        "install" => Ok(ShimCommand::Install { manager_id, dir }),
        "uninstall" => Ok(ShimCommand::Uninstall { manager_id, dir }),
        action => Err(ShimCommandError::UnsupportedAction(action.to_owned())),
    }
}

pub fn install_shim(
    manager_id: &str,
    dir: &Path,
    packvet_executable: &Path,
) -> Result<PathBuf, ShimSetupError> {
    fs::create_dir_all(dir).map_err(|error| ShimSetupError::Io(error.to_string()))?;
    let shim_path = dir.join(manager_id);

    if shim_path.exists() || shim_path.symlink_metadata().is_ok() {
        if is_packvet_shim(&shim_path, packvet_executable) {
            return Ok(shim_path);
        }

        return Err(ShimSetupError::ExistingPath(shim_path));
    }

    create_symlink(packvet_executable, &shim_path)?;
    Ok(shim_path)
}

pub fn uninstall_shim(
    manager_id: &str,
    dir: &Path,
    packvet_executable: &Path,
) -> Result<PathBuf, ShimSetupError> {
    let shim_path = dir.join(manager_id);

    if !shim_path.exists() && shim_path.symlink_metadata().is_err() {
        return Ok(shim_path);
    }

    if !is_packvet_shim(&shim_path, packvet_executable) {
        return Err(ShimSetupError::NotPackvetShim(shim_path));
    }

    fs::remove_file(&shim_path).map_err(|error| ShimSetupError::Io(error.to_string()))?;
    Ok(shim_path)
}

fn is_packvet_shim(path: &Path, packvet_executable: &Path) -> bool {
    path.symlink_metadata()
        .map(|metadata| metadata.file_type().is_symlink())
        .unwrap_or(false)
        && same_file(path, packvet_executable)
}

fn same_file(left: &Path, right: &Path) -> bool {
    match (left.canonicalize(), right.canonicalize()) {
        (Ok(left), Ok(right)) => left == right,
        _ => false,
    }
}

#[cfg(unix)]
fn create_symlink(target: &Path, link: &Path) -> Result<(), ShimSetupError> {
    std::os::unix::fs::symlink(target, link).map_err(|error| ShimSetupError::Io(error.to_string()))
}
