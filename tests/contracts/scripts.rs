use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn run_script(script_name: &str, args: &[&str]) -> std::process::Output {
    Command::new("bash")
        .arg(repo_root().join("scripts").join(script_name))
        .args(args)
        .output()
        .expect("run smoke script")
}

#[test]
fn smoke_local_help_names_safe_default_package() {
    let output = run_script("smoke-local.sh", &["--help"]);

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf-8");
    assert!(stdout.contains("scripts/smoke-local.sh"));
    assert!(stdout.contains("is-number@7.0.0"));
}

#[test]
fn smoke_local_rejects_package_outside_allowlist() {
    let output = run_script("smoke-local.sh", &["left-pad"]);

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).expect("stderr is utf-8");
    assert!(stderr.contains("package is not in the smoke-test allowlist"));
    assert!(stderr.contains("is-number@7.0.0"));
}

#[test]
fn smoke_docker_help_names_safe_default_package() {
    let output = run_script("smoke-docker.sh", &["--help"]);

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf-8");
    assert!(stdout.contains("scripts/smoke-docker.sh"));
    assert!(stdout.contains("is-number@7.0.0"));
}
