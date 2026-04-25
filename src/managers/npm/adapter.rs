use crate::core::{InstallOperation, InstallRequest, InstallTarget, PackageManager};
use crate::core::{ManagerAdapterError, ManagerIntegrationAdapter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NpmManagerAdapter;

impl ManagerIntegrationAdapter for NpmManagerAdapter {
    fn id(&self) -> &'static str {
        "npm"
    }

    fn release_resolver_id(&self) -> &'static str {
        "npm-registry"
    }

    fn release_decision_evaluator_id(&self) -> &'static str {
        "npm-release-policy"
    }

    fn parse_install(&self, args: &[String]) -> Result<InstallRequest, ManagerAdapterError> {
        parse_npm_install(args)
    }
}

fn parse_npm_install(args: &[String]) -> Result<InstallRequest, ManagerAdapterError> {
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
