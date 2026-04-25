use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::Command;
use std::thread;

fn run_lfg(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_lfg"))
        .args(args)
        .output()
        .expect("run lfg binary")
}

fn run_lfg_with_registry_and_now(
    args: &[&str],
    registry_base_url: &str,
    now_unix_seconds: u64,
) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_lfg"))
        .args(args)
        .env("LFG_NPM_REGISTRY_URL", registry_base_url)
        .env("LFG_NOW_UNIX_SECONDS", now_unix_seconds.to_string())
        .output()
        .expect("run lfg binary")
}

fn serve_packument_once(packument: &'static str) -> (String, thread::JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind local test server");
    let address = listener.local_addr().expect("read local server address");
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept one request");
        let mut buffer = [0; 2048];
        let read = stream.read(&mut buffer).expect("read request");
        let request = String::from_utf8_lossy(&buffer[..read]).to_string();
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
            packument.len(),
            packument
        );
        stream
            .write_all(response.as_bytes())
            .expect("write response");
        request
    });

    (format!("http://{address}"), handle)
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
fn explicit_recent_npm_install_fetches_metadata_and_pauses_for_diff_review() {
    let packument = r#"{
      "name": "recent-package",
      "dist-tags": { "latest": "1.1.0" },
      "time": {
        "1.0.0": "1970-01-01T00:00:00.000Z",
        "1.1.0": "1970-01-02T00:00:00.000Z"
      },
      "versions": {
        "1.0.0": {
          "dist": { "tarball": "https://registry.npmjs.org/recent-package/-/recent-package-1.0.0.tgz" }
        },
        "1.1.0": {
          "dist": { "tarball": "https://registry.npmjs.org/recent-package/-/recent-package-1.1.0.tgz" }
        }
      }
    }"#;
    let (registry_base_url, server) = serve_packument_once(packument);

    let output = run_lfg_with_registry_and_now(
        &["npm", "install", "recent-package"],
        &registry_base_url,
        25 * 60 * 60,
    );

    assert_eq!(output.status.code(), Some(20));

    let stderr = String::from_utf8(output.stderr).expect("stderr is utf-8");
    assert_eq!(
        stderr,
        "lfg: review required for npm install, but archive diff review is not wired yet. install is paused.\n"
    );

    let request = server.join().expect("server thread completes");
    assert!(request.starts_with("GET /recent-package HTTP/1.1\r\n"));
}

#[test]
fn explicit_old_npm_install_fetches_metadata_and_pauses_before_execution() {
    let packument = r#"{
      "name": "old-package",
      "dist-tags": { "latest": "1.1.0" },
      "time": {
        "1.0.0": "1970-01-01T00:00:00.000Z",
        "1.1.0": "1970-01-02T00:00:00.000Z"
      },
      "versions": {
        "1.0.0": {
          "dist": { "tarball": "https://registry.npmjs.org/old-package/-/old-package-1.0.0.tgz" }
        },
        "1.1.0": {
          "dist": { "tarball": "https://registry.npmjs.org/old-package/-/old-package-1.1.0.tgz" }
        }
      }
    }"#;
    let (registry_base_url, server) = serve_packument_once(packument);

    let output = run_lfg_with_registry_and_now(
        &["npm", "install", "old-package"],
        &registry_base_url,
        50 * 60 * 60,
    );

    assert_eq!(output.status.code(), Some(20));

    let stderr = String::from_utf8(output.stderr).expect("stderr is utf-8");
    assert_eq!(
        stderr,
        "lfg: npm review is not required by policy, but npm execution is not wired yet. install is paused.\n"
    );

    let request = server.join().expect("server thread completes");
    assert!(request.starts_with("GET /old-package HTTP/1.1\r\n"));
}
