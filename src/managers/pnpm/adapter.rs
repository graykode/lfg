use crate::core::{InstallOperation, InstallRequest, InstallTarget, PackageManager, RealCommand};
use crate::core::{ManagerAdapterError, ManagerIntegrationAdapter};
use crate::managers::package_json::package_json_install_targets;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PnpmManagerAdapter;

impl ManagerIntegrationAdapter for PnpmManagerAdapter {
    fn id(&self) -> &'static str {
        "pnpm"
    }

    fn release_resolver_id(&self) -> &'static str {
        "npm-registry"
    }

    fn release_decision_evaluator_id(&self) -> &'static str {
        "npm-release-policy"
    }

    fn parse_install(&self, args: &[String]) -> Result<InstallRequest, ManagerAdapterError> {
        parse_pnpm_add(args)
    }

    fn real_command(&self, request: &InstallRequest) -> RealCommand {
        RealCommand {
            program: "pnpm".to_owned(),
            args: request.manager_args.clone(),
        }
    }
}

fn parse_pnpm_add(args: &[String]) -> Result<InstallRequest, ManagerAdapterError> {
    let Some(command) = args.first() else {
        return Err(ManagerAdapterError::MissingCommand);
    };

    if command != "add" && command != "install" && command != "i" {
        return Err(ManagerAdapterError::UnsupportedCommand(command.to_owned()));
    }

    let mut targets = Vec::new();
    for arg in args.iter().skip(1) {
        if arg.starts_with('-') {
            if is_allowed_pnpm_add_option(arg) {
                continue;
            }

            return Err(ManagerAdapterError::UnsupportedManagerOption(
                arg.to_owned(),
            ));
        }

        targets.push(InstallTarget {
            spec: arg.to_owned(),
        });
    }

    let operation = if command == "add" {
        InstallOperation::Add
    } else {
        InstallOperation::Install
    };

    if targets.is_empty() && operation == InstallOperation::Add {
        return Err(ManagerAdapterError::MissingPackage);
    }
    if targets.is_empty() {
        targets = package_json_install_targets()?;
    }

    Ok(InstallRequest {
        manager: PackageManager::Pnpm,
        operation,
        targets,
        manager_args: args.to_vec(),
    })
}

fn is_allowed_pnpm_add_option(arg: &str) -> bool {
    matches!(
        arg,
        "-D" | "--save-dev"
            | "-O"
            | "--save-optional"
            | "-P"
            | "--save-prod"
            | "--save-peer"
            | "-E"
            | "--save-exact"
    )
}
