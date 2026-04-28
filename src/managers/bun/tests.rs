use crate::core::{
    InstallOperation, InstallRequest, InstallTarget, ManagerAdapterError,
    ManagerIntegrationAdapter, PackageManager,
};
use crate::managers::bun::BunManagerAdapter;

fn args(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

#[test]
fn parses_bun_add_package() {
    assert_eq!(
        BunManagerAdapter.parse_install(&args(&["add", "left-pad"])),
        Ok(InstallRequest {
            manager: PackageManager::Bun,
            operation: InstallOperation::Add,
            targets: vec![InstallTarget {
                spec: "left-pad".to_owned()
            }],
            manager_args: args(&["add", "left-pad"]),
        })
    );
}

#[test]
fn parses_bun_i_package() {
    let request = BunManagerAdapter
        .parse_install(&args(&["i", "@scope/pkg@1.2.3"]))
        .expect("bun i package should parse");

    assert_eq!(request.manager, PackageManager::Bun);
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
fn builds_real_bun_command_from_original_args() {
    let request = BunManagerAdapter
        .parse_install(&args(&["add", "--dev", "left-pad"]))
        .expect("bun add should parse");
    let command = BunManagerAdapter.real_command(&request);

    assert_eq!(command.program, "bun");
    assert_eq!(command.args, args(&["add", "--dev", "left-pad"]));
}

#[test]
fn asks_on_resolution_affecting_bun_option() {
    assert_eq!(
        BunManagerAdapter.parse_install(&args(&[
            "add",
            "--registry",
            "https://example.invalid",
            "left-pad"
        ])),
        Err(ManagerAdapterError::UnsupportedManagerOption(
            "--registry".to_owned()
        ))
    );
}

#[test]
fn rejects_bun_add_without_package() {
    assert_eq!(
        BunManagerAdapter.parse_install(&args(&["add"])),
        Err(ManagerAdapterError::MissingPackage)
    );
}

#[test]
fn reports_unavailable_package_json_for_bun_install_without_package() {
    assert_eq!(
        BunManagerAdapter.parse_install(&args(&["install"])),
        Err(ManagerAdapterError::ManifestUnavailable(
            "package.json".to_owned()
        ))
    );
}

#[test]
fn rejects_unsupported_bun_command() {
    assert_eq!(
        BunManagerAdapter.parse_install(&args(&["run"])),
        Err(ManagerAdapterError::UnsupportedCommand("run".to_owned()))
    );
}
