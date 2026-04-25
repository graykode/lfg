use crate::adapters::{ManagerAdapterError, ManagerIntegrationAdapter};
use crate::install_request::{InstallOperation, InstallRequest, InstallTarget, PackageManager};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NpmManagerAdapter;

impl ManagerIntegrationAdapter for NpmManagerAdapter {
    fn id(&self) -> &'static str {
        "npm"
    }

    fn parse_install(&self, args: &[String]) -> Result<InstallRequest, ManagerAdapterError> {
        parse_npm_install(args)
    }
}

pub fn parse_npm_install(args: &[String]) -> Result<InstallRequest, ManagerAdapterError> {
    let Some(command) = args.first() else {
        return Err(ManagerAdapterError::MissingCommand);
    };

    if command != "install" && command != "i" {
        return Err(ManagerAdapterError::UnsupportedCommand(command.to_owned()));
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
        return Err(ManagerAdapterError::MissingPackage);
    }

    Ok(InstallRequest {
        manager: PackageManager::Npm,
        operation: InstallOperation::Install,
        targets,
        manager_args: args.to_vec(),
    })
}
