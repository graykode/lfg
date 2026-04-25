use std::fs;

use crate::core::{InstallOperation, InstallRequest, InstallTarget, PackageManager, RealCommand};
use crate::core::{ManagerAdapterError, ManagerIntegrationAdapter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PipManagerAdapter;

impl ManagerIntegrationAdapter for PipManagerAdapter {
    fn id(&self) -> &'static str {
        "pip"
    }

    fn release_resolver_id(&self) -> &'static str {
        "pypi-registry"
    }

    fn release_decision_evaluator_id(&self) -> &'static str {
        "python-release-policy"
    }

    fn parse_install(&self, args: &[String]) -> Result<InstallRequest, ManagerAdapterError> {
        parse_pip_install(args)
    }

    fn real_command(&self, request: &InstallRequest) -> RealCommand {
        RealCommand {
            program: "pip".to_owned(),
            args: request.manager_args.clone(),
        }
    }
}

fn parse_pip_install(args: &[String]) -> Result<InstallRequest, ManagerAdapterError> {
    let Some(command) = args.first() else {
        return Err(ManagerAdapterError::MissingCommand);
    };

    if command != "install" {
        return Err(ManagerAdapterError::UnsupportedCommand(command.to_owned()));
    }

    let mut targets = Vec::new();
    let mut index = 1;
    while index < args.len() {
        let arg = &args[index];

        if arg == "-r" || arg == "--requirement" {
            let Some(path) = args.get(index + 1) else {
                return Err(ManagerAdapterError::MissingRequirementsFile);
            };
            targets.extend(read_requirements_file(path)?);
            index += 2;
            continue;
        }

        if let Some(path) = arg.strip_prefix("--requirement=") {
            targets.extend(read_requirements_file(path)?);
            index += 1;
            continue;
        }

        if arg.starts_with('-') {
            if !is_supported_pip_install_flag(arg) {
                return Err(ManagerAdapterError::UnsupportedManagerOption(
                    arg.to_owned(),
                ));
            }
            index += 1;
            continue;
        }

        targets.push(InstallTarget {
            spec: arg.to_owned(),
        });
        index += 1;
    }

    if targets.is_empty() {
        return Err(ManagerAdapterError::MissingPackage);
    }

    Ok(InstallRequest {
        manager: PackageManager::Pip,
        operation: InstallOperation::Install,
        targets,
        manager_args: args.to_vec(),
    })
}

fn read_requirements_file(path: &str) -> Result<Vec<InstallTarget>, ManagerAdapterError> {
    let content = fs::read_to_string(path)
        .map_err(|_| ManagerAdapterError::RequirementsFileUnavailable(path.to_owned()))?;
    let mut targets = Vec::new();

    for line in content.lines() {
        let line = strip_comment(line).trim();
        if line.is_empty() {
            continue;
        }

        if line.starts_with('-') {
            return Err(ManagerAdapterError::UnsupportedRequirement(line.to_owned()));
        }

        targets.push(InstallTarget {
            spec: line.to_owned(),
        });
    }

    Ok(targets)
}

fn strip_comment(line: &str) -> &str {
    line.split('#').next().unwrap_or(line)
}

fn is_supported_pip_install_flag(arg: &str) -> bool {
    matches!(
        arg,
        "-U" | "--upgrade"
            | "--user"
            | "--no-deps"
            | "--pre"
            | "--force-reinstall"
            | "--ignore-installed"
    )
}
