use lfg::core::{
    InstallOperation, InstallRequest, InstallTarget, ManagerAdapterError,
    ManagerIntegrationAdapter, PackageManager,
};
use lfg::managers::pnpm::PnpmManagerAdapter;

fn args(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

#[test]
fn parses_pnpm_add_package() {
    assert_eq!(
        PnpmManagerAdapter.parse_install(&args(&["add", "left-pad"])),
        Ok(InstallRequest {
            manager: PackageManager::Pnpm,
            operation: InstallOperation::Add,
            targets: vec![InstallTarget {
                spec: "left-pad".to_owned()
            }],
            manager_args: args(&["add", "left-pad"]),
        })
    );
}

#[test]
fn builds_real_pnpm_command_from_original_args() {
    let request = PnpmManagerAdapter
        .parse_install(&args(&["add", "--save-dev", "left-pad"]))
        .expect("pnpm add should parse");
    let command = PnpmManagerAdapter.real_command(&request);

    assert_eq!(command.program, "pnpm");
    assert_eq!(command.args, args(&["add", "--save-dev", "left-pad"]));
}

#[test]
fn asks_on_resolution_affecting_pnpm_option() {
    assert_eq!(
        PnpmManagerAdapter.parse_install(&args(&[
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
fn rejects_pnpm_add_without_package() {
    assert_eq!(
        PnpmManagerAdapter.parse_install(&args(&["add"])),
        Err(ManagerAdapterError::MissingPackage)
    );
}

#[test]
fn rejects_unsupported_pnpm_command() {
    assert_eq!(
        PnpmManagerAdapter.parse_install(&args(&["install"])),
        Err(ManagerAdapterError::UnsupportedCommand(
            "install".to_owned()
        ))
    );
}
