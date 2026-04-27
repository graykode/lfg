use crate::core::{
    InstallOperation, InstallRequest, InstallTarget, ManagerAdapterError,
    ManagerIntegrationAdapter, PackageManager,
};
use crate::managers::uv::UvManagerAdapter;

fn args(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

#[test]
fn parses_uv_add_package() {
    assert_eq!(
        UvManagerAdapter.parse_install(&args(&["add", "requests"])),
        Ok(InstallRequest {
            manager: PackageManager::Uv,
            operation: InstallOperation::Add,
            targets: vec![InstallTarget {
                spec: "requests".to_owned()
            }],
            manager_args: args(&["add", "requests"]),
        })
    );
}

#[test]
fn parses_uv_add_pinned_package() {
    let request = UvManagerAdapter
        .parse_install(&args(&["add", "requests==2.32.3"]))
        .expect("uv add should parse");

    assert_eq!(request.manager, PackageManager::Uv);
    assert_eq!(request.operation, InstallOperation::Add);
    assert_eq!(
        request.targets,
        vec![InstallTarget {
            spec: "requests==2.32.3".to_owned()
        }]
    );
    assert_eq!(request.manager_args, args(&["add", "requests==2.32.3"]));
}

#[test]
fn builds_real_uv_command_from_original_args() {
    let request = UvManagerAdapter
        .parse_install(&args(&["add", "--dev", "pytest"]))
        .expect("uv add should parse");
    let command = UvManagerAdapter.real_command(&request);

    assert_eq!(command.program, "uv");
    assert_eq!(command.args, args(&["add", "--dev", "pytest"]));
}

#[test]
fn asks_on_resolution_affecting_uv_option() {
    assert_eq!(
        UvManagerAdapter.parse_install(&args(&[
            "add",
            "--index",
            "https://example.invalid/simple",
            "requests"
        ])),
        Err(ManagerAdapterError::UnsupportedManagerOption(
            "--index".to_owned()
        ))
    );
}

#[test]
fn rejects_uv_add_without_package() {
    assert_eq!(
        UvManagerAdapter.parse_install(&args(&["add"])),
        Err(ManagerAdapterError::MissingPackage)
    );
}

#[test]
fn rejects_unsupported_uv_command() {
    assert_eq!(
        UvManagerAdapter.parse_install(&args(&["pip", "install", "requests"])),
        Err(ManagerAdapterError::UnsupportedCommand("pip".to_owned()))
    );
}
