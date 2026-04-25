use lfg::core::{
    InstallOperation, InstallRequest, InstallTarget, ManagerAdapterError,
    ManagerIntegrationAdapter, PackageManager,
};
use lfg::managers::npm::NpmManagerAdapter;

fn args(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

#[test]
fn parses_npm_install_package() {
    assert_eq!(
        NpmManagerAdapter.parse_install(&args(&["install", "left-pad"])),
        Ok(InstallRequest {
            manager: PackageManager::Npm,
            operation: InstallOperation::Install,
            targets: vec![InstallTarget {
                spec: "left-pad".to_owned()
            }],
            manager_args: args(&["install", "left-pad"]),
        })
    );
}

#[test]
fn parses_npm_i_alias() {
    let request = NpmManagerAdapter
        .parse_install(&args(&["i", "@scope/pkg@1.2.3"]))
        .expect("npm i alias should parse");

    assert_eq!(request.manager, PackageManager::Npm);
    assert_eq!(request.operation, InstallOperation::Install);
    assert_eq!(
        request.targets,
        vec![InstallTarget {
            spec: "@scope/pkg@1.2.3".to_owned()
        }]
    );
    assert_eq!(request.manager_args, args(&["i", "@scope/pkg@1.2.3"]));
}

#[test]
fn rejects_npm_install_without_package() {
    assert_eq!(
        NpmManagerAdapter.parse_install(&args(&["install"])),
        Err(ManagerAdapterError::MissingPackage)
    );
}

#[test]
fn rejects_unsupported_npm_command() {
    assert_eq!(
        NpmManagerAdapter.parse_install(&args(&["run", "build"])),
        Err(ManagerAdapterError::UnsupportedCommand("run".to_owned()))
    );
}
