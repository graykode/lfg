use std::fs;

use crate::cli::support::{
    path_with_fake_bin, run_packvet_in_dir_with_registry_now_and_env,
    run_packvet_with_registry_now_and_env, serve_packument_once, temp_test_dir, write_fake_npm_bin,
};

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
    let temp_dir = temp_test_dir("packvet-fake-npm");
    let fake_bin_dir = temp_dir.join("bin");
    let fake_args_path = temp_dir.join("npm-args.txt");
    write_fake_npm_bin(&fake_bin_dir);

    let output = run_packvet_with_registry_now_and_env(
        &["npm", "install", "old-package"],
        &registry_base_url,
        50 * 60 * 60,
        &[
            ("PATH", path_with_fake_bin(&fake_bin_dir)),
            (
                "PACKVET_FAKE_NPM_ARGS",
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
    assert_eq!(
        stderr,
        "\
packvet: checking npm install old-package
packvet: resolving npm metadata for old-package
packvet: skipped review for old-package; older than configured threshold
packvet: running npm install old-package
fake npm stderr
"
    );
    assert_eq!(
        fs::read_to_string(&fake_args_path).expect("fake npm args are captured"),
        "install\nold-package\n"
    );

    let request = server.join().expect("server thread completes");
    assert!(request.starts_with("GET /old-package HTTP/1.1\r\n"));

    fs::remove_dir_all(temp_dir).expect("remove fake npm temp dir");
}

#[test]
fn npm_i_without_package_reads_package_json_and_executes_real_npm_after_policy_pass() {
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
    let temp_dir = temp_test_dir("packvet-npm-package-json");
    fs::create_dir_all(&temp_dir).expect("create npm package temp dir");
    fs::write(
        temp_dir.join("package.json"),
        r#"{"dependencies":{"old-package":"1.1.0"}}"#,
    )
    .expect("write package.json");
    let fake_bin_dir = temp_dir.join("bin");
    let fake_args_path = temp_dir.join("npm-args.txt");
    write_fake_npm_bin(&fake_bin_dir);

    let output = run_packvet_in_dir_with_registry_now_and_env(
        &temp_dir,
        &["npm", "i"],
        &registry_base_url,
        50 * 60 * 60,
        &[
            ("PATH", path_with_fake_bin(&fake_bin_dir)),
            (
                "PACKVET_FAKE_NPM_ARGS",
                fake_args_path.to_string_lossy().into_owned(),
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
        "\
packvet: checking npm i
packvet: resolving npm metadata for old-package@1.1.0
packvet: skipped review for old-package@1.1.0; older than configured threshold
packvet: running npm i
fake npm stderr
"
    );
    assert_eq!(
        fs::read_to_string(&fake_args_path).expect("fake npm args are captured"),
        "i\n"
    );

    let request = server.join().expect("server thread completes");
    assert!(request.starts_with("GET /old-package HTTP/1.1\r\n"));

    fs::remove_dir_all(temp_dir).expect("remove npm package temp dir");
}

#[test]
fn npm_i_without_package_pauses_when_package_json_is_missing() {
    let temp_dir = temp_test_dir("packvet-npm-missing-package-json");
    fs::create_dir_all(&temp_dir).expect("create npm missing package temp dir");

    let output = run_packvet_in_dir_with_registry_now_and_env(
        &temp_dir,
        &["npm", "i"],
        "http://127.0.0.1:9",
        50 * 60 * 60,
        &[],
    );

    assert_eq!(output.status.code(), Some(20));
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr is utf-8"),
        "packvet: npm package manifest is unavailable: package.json; install is paused.\n"
    );

    fs::remove_dir_all(temp_dir).expect("remove npm missing package temp dir");
}
