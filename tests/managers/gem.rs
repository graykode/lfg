use lfg::core::{
    InstallOperation, InstallRequest, InstallTarget, ManagerAdapterError,
    ManagerIntegrationAdapter, PackageManager,
};
use lfg::managers::gem::GemManagerAdapter;

fn args(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

#[test]
fn parses_gem_install_package() {
    assert_eq!(
        GemManagerAdapter.parse_install(&args(&["install", "rack"])),
        Ok(InstallRequest {
            manager: PackageManager::Gem,
            operation: InstallOperation::Install,
            targets: vec![InstallTarget {
                spec: "rack".to_owned()
            }],
            manager_args: args(&["install", "rack"]),
        })
    );
}

#[test]
fn parses_gem_install_exact_version() {
    let request = GemManagerAdapter
        .parse_install(&args(&["install", "rack", "--version", "3.0.0"]))
        .expect("gem install exact version should parse");

    assert_eq!(
        request.targets,
        vec![InstallTarget {
            spec: "rack@3.0.0".to_owned()
        }]
    );
}

#[test]
fn builds_real_gem_command_from_original_args() {
    let request = GemManagerAdapter
        .parse_install(&args(&["install", "--no-document", "rack"]))
        .expect("gem install should parse");
    let command = GemManagerAdapter.real_command(&request);

    assert_eq!(command.program, "gem");
    assert_eq!(command.args, args(&["install", "--no-document", "rack"]));
}

#[test]
fn asks_on_resolution_affecting_gem_option() {
    assert_eq!(
        GemManagerAdapter.parse_install(&args(&[
            "install",
            "--source",
            "https://example.invalid",
            "rack"
        ])),
        Err(ManagerAdapterError::UnsupportedManagerOption(
            "--source".to_owned()
        ))
    );
}

#[test]
fn rejects_gem_install_without_package() {
    assert_eq!(
        GemManagerAdapter.parse_install(&args(&["install"])),
        Err(ManagerAdapterError::MissingPackage)
    );
}

#[test]
fn rejects_unsupported_gem_command() {
    assert_eq!(
        GemManagerAdapter.parse_install(&args(&["update", "rack"])),
        Err(ManagerAdapterError::UnsupportedCommand("update".to_owned()))
    );
}
