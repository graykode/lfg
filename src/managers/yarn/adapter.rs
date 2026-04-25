use crate::core::{InstallOperation, InstallRequest, InstallTarget, PackageManager, RealCommand};
use crate::core::{ManagerAdapterError, ManagerIntegrationAdapter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct YarnManagerAdapter;

impl ManagerIntegrationAdapter for YarnManagerAdapter {
    fn id(&self) -> &'static str {
        "yarn"
    }

    fn release_resolver_id(&self) -> &'static str {
        "npm-registry"
    }

    fn release_decision_evaluator_id(&self) -> &'static str {
        "npm-release-policy"
    }

    fn parse_install(&self, args: &[String]) -> Result<InstallRequest, ManagerAdapterError> {
        parse_yarn_add(args)
    }

    fn real_command(&self, request: &InstallRequest) -> RealCommand {
        RealCommand {
            program: "yarn".to_owned(),
            args: request.manager_args.clone(),
        }
    }
}

fn parse_yarn_add(args: &[String]) -> Result<InstallRequest, ManagerAdapterError> {
    let Some(command) = args.first() else {
        return Err(ManagerAdapterError::MissingCommand);
    };

    if command != "add" {
        return Err(ManagerAdapterError::UnsupportedCommand(command.to_owned()));
    }

    let mut targets = Vec::new();
    for arg in args.iter().skip(1) {
        if arg.starts_with('-') {
            if is_allowed_yarn_add_option(arg) {
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

    if targets.is_empty() {
        return Err(ManagerAdapterError::MissingPackage);
    }

    Ok(InstallRequest {
        manager: PackageManager::Yarn,
        operation: InstallOperation::Add,
        targets,
        manager_args: args.to_vec(),
    })
}

fn is_allowed_yarn_add_option(arg: &str) -> bool {
    matches!(
        arg,
        "-D" | "--dev"
            | "-P"
            | "--peer"
            | "-O"
            | "--optional"
            | "-E"
            | "--exact"
            | "-T"
            | "--tilde"
    )
}
