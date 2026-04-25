use lfg::core::{
    InstallOperation, InstallRequest, InstallTarget, ManagerAdapterError,
    ManagerIntegrationAdapter, PackageManager,
};
use lfg::managers::cargo::CargoManagerAdapter;

fn args(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

#[test]
fn parses_cargo_add_crate() {
    assert_eq!(
        CargoManagerAdapter.parse_install(&args(&["add", "serde"])),
        Ok(InstallRequest {
            manager: PackageManager::Cargo,
            operation: InstallOperation::Add,
            targets: vec![InstallTarget {
                spec: "serde".to_owned()
            }],
            manager_args: args(&["add", "serde"]),
        })
    );
}

#[test]
fn parses_cargo_add_pinned_crate() {
    let request = CargoManagerAdapter
        .parse_install(&args(&["add", "serde@1.0.228"]))
        .expect("cargo add pinned crate should parse");

    assert_eq!(
        request.targets,
        vec![InstallTarget {
            spec: "serde@1.0.228".to_owned()
        }]
    );
}

#[test]
fn builds_real_cargo_command_from_original_args() {
    let request = CargoManagerAdapter
        .parse_install(&args(&["add", "--dev", "serde"]))
        .expect("cargo add should parse");
    let command = CargoManagerAdapter.real_command(&request);

    assert_eq!(command.program, "cargo");
    assert_eq!(command.args, args(&["add", "--dev", "serde"]));
}

#[test]
fn asks_on_resolution_affecting_cargo_option() {
    assert_eq!(
        CargoManagerAdapter.parse_install(&args(&[
            "add",
            "--git",
            "https://example.invalid/repo.git"
        ])),
        Err(ManagerAdapterError::UnsupportedManagerOption(
            "--git".to_owned()
        ))
    );
}

#[test]
fn rejects_cargo_add_without_crate() {
    assert_eq!(
        CargoManagerAdapter.parse_install(&args(&["add"])),
        Err(ManagerAdapterError::MissingPackage)
    );
}

#[test]
fn rejects_unsupported_cargo_command() {
    assert_eq!(
        CargoManagerAdapter.parse_install(&args(&["build"])),
        Err(ManagerAdapterError::UnsupportedCommand("build".to_owned()))
    );
}
