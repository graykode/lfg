use std::collections::BTreeMap;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};
use std::time::{SystemTime, UNIX_EPOCH};

use flate2::write::GzEncoder;
use flate2::Compression;
use tar::{Builder, Header};

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
        .env("LFG_REVIEW_PROVIDER", "none")
        .output()
        .expect("run lfg binary")
}

fn run_lfg_with_registry_now_and_env(
    args: &[&str],
    registry_base_url: &str,
    now_unix_seconds: u64,
    envs: &[(&str, String)],
) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_lfg"));
    command
        .args(args)
        .env("LFG_NPM_REGISTRY_URL", registry_base_url)
        .env("LFG_NOW_UNIX_SECONDS", now_unix_seconds.to_string())
        .env("LFG_REVIEW_PROVIDER", "none");

    for (key, value) in envs {
        command.env(key, value);
    }

    command.output().expect("run lfg binary")
}

fn temp_test_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}-{nanos}"))
}

fn write_fake_npm_bin(dir: &Path) {
    fs::create_dir_all(dir).expect("create fake bin dir");
    let npm_path = dir.join("npm");
    fs::write(
        &npm_path,
        "#!/bin/sh\nprintf '%s\\n' \"$@\" > \"$LFG_FAKE_NPM_ARGS\"\nprintf 'fake npm stdout\\n'\nprintf 'fake npm stderr\\n' >&2\n",
    )
    .expect("write fake npm");
    let mut permissions = fs::metadata(&npm_path)
        .expect("read fake npm metadata")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(npm_path, permissions).expect("mark fake npm executable");
}

fn write_fake_claude_bin(dir: &Path, provider_output: &str) {
    fs::create_dir_all(dir).expect("create fake bin dir");
    let claude_path = dir.join("claude");
    fs::write(
        &claude_path,
        format!(
            "#!/bin/sh\ncat > \"$LFG_FAKE_PROVIDER_PROMPT\"\ncat <<'LFG_PROVIDER_OUTPUT'\n{provider_output}LFG_PROVIDER_OUTPUT\n"
        ),
    )
    .expect("write fake claude");
    let mut permissions = fs::metadata(&claude_path)
        .expect("read fake claude metadata")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(claude_path, permissions).expect("mark fake claude executable");
}

fn path_with_fake_bin(bin_dir: &Path) -> String {
    let existing = std::env::var_os("PATH").unwrap_or_default();
    let paths = std::iter::once(bin_dir.to_path_buf()).chain(std::env::split_paths(&existing));
    std::env::join_paths(paths)
        .expect("join PATH")
        .to_string_lossy()
        .into_owned()
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

fn tgz(entries: &[(&str, &str)]) -> Vec<u8> {
    let mut tar_bytes = Vec::new();
    {
        let mut builder = Builder::new(&mut tar_bytes);
        for (path, content) in entries {
            let mut header = Header::new_gnu();
            header.set_path(path).expect("set tar path");
            header.set_size(content.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            builder
                .append(&header, content.as_bytes())
                .expect("append tar entry");
        }
        builder.finish().expect("finish tar");
    }

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&tar_bytes).expect("write gzip body");
    encoder.finish().expect("finish gzip")
}

fn serve_recent_package_with_archives() -> (String, thread::JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind local test server");
    listener
        .set_nonblocking(true)
        .expect("configure nonblocking listener");
    let address = listener.local_addr().expect("read local server address");
    let registry_base_url = format!("http://{address}");
    let packument = format!(
        r#"{{
      "name": "recent-package",
      "dist-tags": {{ "latest": "1.1.0" }},
      "time": {{
        "1.0.0": "1970-01-01T00:00:00.000Z",
        "1.1.0": "1970-01-02T00:00:00.000Z"
      }},
      "versions": {{
        "1.0.0": {{
          "dist": {{ "tarball": "{registry_base_url}/recent-package-1.0.0.tgz" }}
        }},
        "1.1.0": {{
          "dist": {{ "tarball": "{registry_base_url}/recent-package-1.1.0.tgz" }}
        }}
      }}
    }}"#
    );
    let responses = BTreeMap::from([
        (
            "/recent-package".to_owned(),
            ("application/json", packument.into_bytes()),
        ),
        (
            "/recent-package-1.0.0.tgz".to_owned(),
            (
                "application/octet-stream",
                tgz(&[("package/index.js", "module.exports = 1;\n")]),
            ),
        ),
        (
            "/recent-package-1.1.0.tgz".to_owned(),
            (
                "application/octet-stream",
                tgz(&[("package/index.js", "module.exports = 2;\n")]),
            ),
        ),
    ]);

    let handle = thread::spawn(move || {
        let mut requests = Vec::new();
        let started_at = Instant::now();
        let mut last_request_at = Instant::now();

        loop {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    stream
                        .set_nonblocking(false)
                        .expect("configure blocking stream");
                    let mut buffer = [0; 2048];
                    let read = stream.read(&mut buffer).expect("read request");
                    let request = String::from_utf8_lossy(&buffer[..read]).to_string();
                    let path = request
                        .lines()
                        .next()
                        .and_then(|line| line.split_whitespace().nth(1))
                        .expect("request path")
                        .to_owned();
                    let (content_type, body) = responses
                        .get(&path)
                        .unwrap_or_else(|| panic!("unexpected request path: {path}"));
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\n\r\n",
                        body.len()
                    );
                    stream
                        .write_all(response.as_bytes())
                        .expect("write response header");
                    stream.write_all(body).expect("write response body");
                    requests.push(request);
                    last_request_at = Instant::now();
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    if requests.len() >= 3
                        || (!requests.is_empty()
                            && last_request_at.elapsed() > Duration::from_millis(100))
                        || started_at.elapsed() > Duration::from_secs(2)
                    {
                        break;
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(error) => panic!("accept request: {error}"),
            }
        }

        requests
    });

    (registry_base_url, handle)
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
    let (registry_base_url, server) = serve_recent_package_with_archives();

    let output = run_lfg_with_registry_and_now(
        &["npm", "install", "recent-package"],
        &registry_base_url,
        25 * 60 * 60,
    );

    assert_eq!(output.status.code(), Some(20));

    let stderr = String::from_utf8(output.stderr).expect("stderr is utf-8");
    assert_eq!(
        stderr,
        "lfg: review required for npm install, but provider review is not wired yet. install is paused.\n"
    );

    let requests = server.join().expect("server thread completes");
    assert_eq!(requests.len(), 3);
    assert!(requests[0].starts_with("GET /recent-package HTTP/1.1\r\n"));
    assert!(requests
        .iter()
        .any(|request| request.starts_with("GET /recent-package-1.0.0.tgz HTTP/1.1\r\n")));
    assert!(requests
        .iter()
        .any(|request| request.starts_with("GET /recent-package-1.1.0.tgz HTTP/1.1\r\n")));
}

#[test]
fn explicit_recent_npm_install_executes_real_npm_after_provider_pass() {
    let (registry_base_url, server) = serve_recent_package_with_archives();
    let temp_dir = temp_test_dir("lfg-fake-provider-pass");
    let fake_bin_dir = temp_dir.join("bin");
    let fake_args_path = temp_dir.join("npm-args.txt");
    let fake_prompt_path = temp_dir.join("provider-prompt.txt");
    write_fake_npm_bin(&fake_bin_dir);
    write_fake_claude_bin(
        &fake_bin_dir,
        "verdict: pass\nreason: fixture allowed\n\nevidence:\n- package/index.js: fixture signal\n",
    );

    let output = run_lfg_with_registry_now_and_env(
        &["npm", "install", "recent-package"],
        &registry_base_url,
        25 * 60 * 60,
        &[
            ("PATH", path_with_fake_bin(&fake_bin_dir)),
            ("LFG_REVIEW_PROVIDER", "claude".to_owned()),
            (
                "LFG_FAKE_NPM_ARGS",
                fake_args_path.to_string_lossy().into_owned(),
            ),
            (
                "LFG_FAKE_PROVIDER_PROMPT",
                fake_prompt_path.to_string_lossy().into_owned(),
            ),
        ],
    );

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout is utf-8"),
        "fake npm stdout\n"
    );
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr is utf-8"),
        "fake npm stderr\n"
    );
    assert_eq!(
        fs::read_to_string(&fake_args_path).expect("fake npm args are captured"),
        "install\nrecent-package\n"
    );
    let prompt = fs::read_to_string(&fake_prompt_path).expect("provider prompt is captured");
    assert!(prompt.contains("package: recent-package"));
    assert!(prompt.contains("previous version: 1.0.0"));
    assert!(prompt.contains("target version: 1.1.0"));
    assert!(prompt.contains("+module.exports = 2;"));

    let requests = server.join().expect("server thread completes");
    assert_eq!(requests.len(), 3);

    fs::remove_dir_all(temp_dir).expect("remove fake provider temp dir");
}

#[test]
fn explicit_recent_npm_install_does_not_execute_real_npm_after_provider_block() {
    let (registry_base_url, server) = serve_recent_package_with_archives();
    let temp_dir = temp_test_dir("lfg-fake-provider-block");
    let fake_bin_dir = temp_dir.join("bin");
    let fake_args_path = temp_dir.join("npm-args.txt");
    let fake_prompt_path = temp_dir.join("provider-prompt.txt");
    write_fake_npm_bin(&fake_bin_dir);
    write_fake_claude_bin(
        &fake_bin_dir,
        "verdict: block\nreason: fixture blocked\n\nevidence:\n- package/index.js: fixture signal\n",
    );

    let output = run_lfg_with_registry_now_and_env(
        &["npm", "install", "recent-package"],
        &registry_base_url,
        25 * 60 * 60,
        &[
            ("PATH", path_with_fake_bin(&fake_bin_dir)),
            ("LFG_REVIEW_PROVIDER", "claude".to_owned()),
            (
                "LFG_FAKE_NPM_ARGS",
                fake_args_path.to_string_lossy().into_owned(),
            ),
            (
                "LFG_FAKE_PROVIDER_PROMPT",
                fake_prompt_path.to_string_lossy().into_owned(),
            ),
        ],
    );

    assert_eq!(output.status.code(), Some(30));
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr is utf-8"),
        "lfg: npm install was blocked by provider review.\n"
    );
    assert!(!fake_args_path.exists());
    let prompt = fs::read_to_string(&fake_prompt_path).expect("provider prompt is captured");
    assert!(prompt.contains("package: recent-package"));
    assert!(prompt.contains("+module.exports = 2;"));

    let requests = server.join().expect("server thread completes");
    assert_eq!(requests.len(), 3);

    fs::remove_dir_all(temp_dir).expect("remove fake provider temp dir");
}

#[test]
fn explicit_old_npm_install_executes_real_npm_after_policy_pass() {
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
    let temp_dir = temp_test_dir("lfg-fake-npm");
    let fake_bin_dir = temp_dir.join("bin");
    let fake_args_path = temp_dir.join("npm-args.txt");
    write_fake_npm_bin(&fake_bin_dir);

    let output = run_lfg_with_registry_now_and_env(
        &["npm", "install", "old-package"],
        &registry_base_url,
        50 * 60 * 60,
        &[
            ("PATH", path_with_fake_bin(&fake_bin_dir)),
            (
                "LFG_FAKE_NPM_ARGS",
                fake_args_path.to_string_lossy().into_owned(),
            ),
        ],
    );

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout is utf-8"),
        "fake npm stdout\n"
    );
    let stderr = String::from_utf8(output.stderr).expect("stderr is utf-8");
    assert_eq!(stderr, "fake npm stderr\n");
    assert_eq!(
        fs::read_to_string(&fake_args_path).expect("fake npm args are captured"),
        "install\nold-package\n"
    );

    let request = server.join().expect("server thread completes");
    assert!(request.starts_with("GET /old-package HTTP/1.1\r\n"));

    fs::remove_dir_all(temp_dir).expect("remove fake npm temp dir");
}
