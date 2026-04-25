use lfg::core::{
    InstallOperation, InstallRequest, InstallTarget, ManagerAdapterError,
    ManagerIntegrationAdapter, PackageManager,
};
use lfg::managers::pip::PipManagerAdapter;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

fn args(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

fn temp_requirements_file(content: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("lfg-requirements-{nanos}.txt"));
    fs::write(&path, content).expect("write requirements file");
    path.to_string_lossy().into_owned()
}

#[test]
fn parses_pip_install_package() {
    assert_eq!(
        PipManagerAdapter.parse_install(&args(&["install", "requests"])),
        Ok(InstallRequest {
            manager: PackageManager::Pip,
            operation: InstallOperation::Install,
            targets: vec![InstallTarget {
                spec: "requests".to_owned()
            }],
            manager_args: args(&["install", "requests"]),
        })
    );
}

#[test]
fn parses_pip_install_pinned_package() {
    let request = PipManagerAdapter
        .parse_install(&args(&["install", "requests==2.32.3"]))
        .expect("pip install should parse");

    assert_eq!(request.manager, PackageManager::Pip);
    assert_eq!(request.operation, InstallOperation::Install);
    assert_eq!(
        request.targets,
        vec![InstallTarget {
            spec: "requests==2.32.3".to_owned()
        }]
    );
    assert_eq!(request.manager_args, args(&["install", "requests==2.32.3"]));
}

#[test]
fn builds_real_pip_command_from_original_args() {
    let request = PipManagerAdapter
        .parse_install(&args(&["install", "--upgrade", "requests"]))
        .expect("pip install should parse");
    let command = PipManagerAdapter.real_command(&request);

    assert_eq!(command.program, "pip");
    assert_eq!(command.args, args(&["install", "--upgrade", "requests"]));
}

#[test]
fn asks_on_resolution_affecting_pip_option() {
    assert_eq!(
        PipManagerAdapter.parse_install(&args(&[
            "install",
            "--index-url",
            "https://example.invalid/simple",
            "requests"
        ])),
        Err(ManagerAdapterError::UnsupportedManagerOption(
            "--index-url".to_owned()
        ))
    );
}

#[test]
fn parses_pip_install_requirements_file() {
    let requirements_path = temp_requirements_file(
        "\
# comment
requests==2.32.3

urllib3
",
    );

    let request = PipManagerAdapter
        .parse_install(&args(&["install", "-r", &requirements_path]))
        .expect("pip install -r should parse");

    assert_eq!(
        request.targets,
        vec![
            InstallTarget {
                spec: "requests==2.32.3".to_owned()
            },
            InstallTarget {
                spec: "urllib3".to_owned()
            },
        ]
    );
    assert_eq!(
        request.manager_args,
        args(&["install", "-r", &requirements_path])
    );

    fs::remove_file(requirements_path).expect("remove requirements file");
}

#[test]
fn rejects_pip_install_requirement_flag_without_path() {
    assert_eq!(
        PipManagerAdapter.parse_install(&args(&["install", "-r"])),
        Err(ManagerAdapterError::MissingRequirementsFile)
    );
}

#[test]
fn reports_unavailable_requirements_file() {
    assert_eq!(
        PipManagerAdapter.parse_install(&args(&[
            "install",
            "-r",
            "/tmp/lfg-missing-requirements.txt"
        ])),
        Err(ManagerAdapterError::RequirementsFileUnavailable(
            "/tmp/lfg-missing-requirements.txt".to_owned()
        ))
    );
}

#[test]
fn asks_on_unsupported_requirements_file_line() {
    let requirements_path = temp_requirements_file(
        "\
--index-url https://example.invalid/simple
requests
",
    );

    assert_eq!(
        PipManagerAdapter.parse_install(&args(&["install", "-r", &requirements_path])),
        Err(ManagerAdapterError::UnsupportedRequirement(
            "--index-url https://example.invalid/simple".to_owned()
        ))
    );

    fs::remove_file(requirements_path).expect("remove requirements file");
}

#[test]
fn rejects_pip_install_without_package() {
    assert_eq!(
        PipManagerAdapter.parse_install(&args(&["install"])),
        Err(ManagerAdapterError::MissingPackage)
    );
}

#[test]
fn rejects_unsupported_pip_command() {
    assert_eq!(
        PipManagerAdapter.parse_install(&args(&["list"])),
        Err(ManagerAdapterError::UnsupportedCommand("list".to_owned()))
    );
}
