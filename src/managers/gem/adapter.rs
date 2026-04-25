use crate::core::{InstallOperation, InstallRequest, InstallTarget, PackageManager, RealCommand};
use crate::core::{ManagerAdapterError, ManagerIntegrationAdapter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GemManagerAdapter;

impl ManagerIntegrationAdapter for GemManagerAdapter {
    fn id(&self) -> &'static str {
        "gem"
    }

    fn release_resolver_id(&self) -> &'static str {
        "rubygems-registry"
    }

    fn release_decision_evaluator_id(&self) -> &'static str {
        "ruby-release-policy"
    }

    fn parse_install(&self, args: &[String]) -> Result<InstallRequest, ManagerAdapterError> {
        parse_gem_install(args)
    }

    fn real_command(&self, request: &InstallRequest) -> RealCommand {
        RealCommand {
            program: "gem".to_owned(),
            args: request.manager_args.clone(),
        }
    }
}

fn parse_gem_install(args: &[String]) -> Result<InstallRequest, ManagerAdapterError> {
    let Some(command) = args.first() else {
        return Err(ManagerAdapterError::MissingCommand);
    };

    if command != "install" {
        return Err(ManagerAdapterError::UnsupportedCommand(command.to_owned()));
    }

    let mut targets = Vec::new();
    let mut exact_version: Option<String> = None;
    let mut index = 1;
    while index < args.len() {
        let arg = &args[index];
        if arg == "--version" || arg == "-v" {
            let Some(version) = args.get(index + 1) else {
                return Err(ManagerAdapterError::UnsupportedManagerOption(
                    arg.to_owned(),
                ));
            };
            if !is_exact_gem_version(version) {
                return Err(ManagerAdapterError::UnsupportedManagerOption(
                    arg.to_owned(),
                ));
            }
            exact_version = Some(version.to_owned());
            index += 2;
            continue;
        }

        if let Some(version) = arg.strip_prefix("--version=") {
            if !is_exact_gem_version(version) {
                return Err(ManagerAdapterError::UnsupportedManagerOption(
                    "--version".to_owned(),
                ));
            }
            exact_version = Some(version.to_owned());
            index += 1;
            continue;
        }

        if arg.starts_with('-') {
            if is_allowed_gem_install_option(arg) {
                index += 1;
                continue;
            }

            return Err(ManagerAdapterError::UnsupportedManagerOption(
                arg.to_owned(),
            ));
        }

        targets.push(arg.to_owned());
        index += 1;
    }

    if targets.is_empty() {
        return Err(ManagerAdapterError::MissingPackage);
    }

    let targets = match exact_version {
        Some(version) => {
            if targets.len() != 1 {
                return Err(ManagerAdapterError::UnsupportedManagerOption(
                    "--version".to_owned(),
                ));
            }

            vec![InstallTarget {
                spec: format!("{}@{}", targets[0], version),
            }]
        }
        None => targets
            .into_iter()
            .map(|spec| InstallTarget { spec })
            .collect(),
    };

    Ok(InstallRequest {
        manager: PackageManager::Gem,
        operation: InstallOperation::Install,
        targets,
        manager_args: args.to_vec(),
    })
}

fn is_allowed_gem_install_option(arg: &str) -> bool {
    matches!(arg, "--no-document" | "--user-install")
}

fn is_exact_gem_version(version: &str) -> bool {
    !version.trim().is_empty() && !version.contains(['<', '>', '~', '!', '='])
}
