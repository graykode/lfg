use std::process::Command;

fn run_lfg(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_lfg"))
        .args(args)
        .output()
        .expect("run lfg binary")
}

#[test]
fn no_arguments_exits_with_ask() {
    let output = run_lfg(&[]);

    assert_eq!(output.status.code(), Some(20));
}

#[test]
fn help_exits_successfully() {
    let output = run_lfg(&["--help"]);

    assert_eq!(output.status.code(), Some(0));

    let stdout = String::from_utf8(output.stdout).expect("stdout is utf-8");
    assert!(stdout.contains("Usage: lfg"));
    assert!(stdout.contains("lfg is a local pre-install guard"));
}

#[test]
fn version_exits_successfully() {
    let output = run_lfg(&["--version"]);

    assert_eq!(output.status.code(), Some(0));

    let stdout = String::from_utf8(output.stdout).expect("stdout is utf-8");
    assert!(stdout.trim().starts_with("lfg "));
}

#[test]
fn unknown_argument_is_cli_misuse() {
    let output = run_lfg(&["--definitely-unknown"]);

    assert_eq!(output.status.code(), Some(1));

    let stderr = String::from_utf8(output.stderr).expect("stderr is utf-8");
    assert!(stderr.contains("unknown argument: --definitely-unknown"));
}

#[test]
fn explicit_npm_install_pauses_until_review_is_available() {
    let output = run_lfg(&["npm", "install", "left-pad"]);

    assert_eq!(output.status.code(), Some(20));

    let stderr = String::from_utf8(output.stderr).expect("stderr is utf-8");
    assert_eq!(
        stderr,
        "lfg: npm install review is not wired yet, so install is paused.\n"
    );
}
