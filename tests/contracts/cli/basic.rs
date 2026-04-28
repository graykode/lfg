use super::support::run_packvet;

#[test]
fn no_arguments_exits_with_ask() {
    let output = run_packvet(&[]);

    assert_eq!(output.status.code(), Some(20));
}

#[test]
fn help_exits_successfully() {
    let output = run_packvet(&["--help"]);

    assert_eq!(output.status.code(), Some(0));

    let stdout = String::from_utf8(output.stdout).expect("stdout is utf-8");
    assert!(stdout.contains("Usage: packvet"));
    assert!(stdout.contains("packvet is a local pre-install guard"));
}

#[test]
fn version_exits_successfully() {
    let output = run_packvet(&["--version"]);

    assert_eq!(output.status.code(), Some(0));

    let stdout = String::from_utf8(output.stdout).expect("stdout is utf-8");
    assert!(stdout.trim().starts_with("packvet "));
}

#[test]
fn unknown_argument_is_cli_misuse() {
    let output = run_packvet(&["--definitely-unknown"]);

    assert_eq!(output.status.code(), Some(1));

    let stderr = String::from_utf8(output.stderr).expect("stderr is utf-8");
    assert!(stderr.contains("unknown argument: --definitely-unknown"));
}
