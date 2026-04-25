use crate::core::{InstallOperation, InstallRequest, InstallTarget, PackageManager, RealCommand};
use crate::core::{ManagerAdapterError, ManagerIntegrationAdapter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UvManagerAdapter;

impl ManagerIntegrationAdapter for UvManagerAdapter {
    fn id(&self) -> &'static str {
        "uv"
    }

    fn release_resolver_id(&self) -> &'static str {
        "pypi-registry"
    }

    fn release_decision_evaluator_id(&self) -> &'static str {
        "python-release-policy"
    }

    fn parse_install(&self, args: &[String]) -> Result<InstallRequest, ManagerAdapterError> {
        parse_uv_add(args)
    }

    fn real_command(&self, request: &InstallRequest) -> RealCommand {
        RealCommand {
            program: "uv".to_owned(),
            args: request.manager_args.clone(),
        }
    }
}

fn parse_uv_add(args: &[String]) -> Result<InstallRequest, ManagerAdapterError> {
    let Some(command) = args.first() else {
        return Err(ManagerAdapterError::MissingCommand);
    };

    if command != "add" {
        return Err(ManagerAdapterError::UnsupportedCommand(command.to_owned()));
    }

    let mut targets = Vec::new();
    for arg in args.iter().skip(1) {
        if arg.starts_with('-') {
            if !is_supported_uv_add_flag(arg) {
                return Err(ManagerAdapterError::UnsupportedManagerOption(
                    arg.to_owned(),
                ));
            }
            continue;
        }

        targets.push(InstallTarget {
            spec: arg.to_owned(),
        });
    }

    if targets.is_empty() {
        return Err(ManagerAdapterError::MissingPackage);
    }

    Ok(InstallRequest {
        manager: PackageManager::Uv,
        operation: InstallOperation::Add,
        targets,
        manager_args: args.to_vec(),
    })
}

fn is_supported_uv_add_flag(arg: &str) -> bool {
    matches!(arg, "--dev")
}
