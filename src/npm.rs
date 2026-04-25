use crate::install_request::{InstallOperation, InstallRequest, InstallTarget, PackageManager};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NpmParseError {
    MissingCommand,
    MissingPackage,
    UnsupportedCommand(String),
}

pub fn parse_npm_install(args: &[String]) -> Result<InstallRequest, NpmParseError> {
    let Some(command) = args.first() else {
        return Err(NpmParseError::MissingCommand);
    };

    if command != "install" && command != "i" {
        return Err(NpmParseError::UnsupportedCommand(command.to_owned()));
    }

    let targets: Vec<InstallTarget> = args
        .iter()
        .skip(1)
        .filter(|arg| !arg.starts_with('-'))
        .map(|spec| InstallTarget {
            spec: spec.to_owned(),
        })
        .collect();

    if targets.is_empty() {
        return Err(NpmParseError::MissingPackage);
    }

    Ok(InstallRequest {
        manager: PackageManager::Npm,
        operation: InstallOperation::Install,
        targets,
        manager_args: args.to_vec(),
    })
}
