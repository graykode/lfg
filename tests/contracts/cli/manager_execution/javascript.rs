use std::fs;

use crate::cli::support::{
    path_with_fake_bin, run_packvet_in_dir_with_registry_now_and_env,
    run_packvet_with_registry_now_and_env, serve_packument_once, temp_test_dir, write_fake_bun_bin,
    write_fake_pnpm_bin, write_fake_yarn_bin,
};

const OLD_PACKAGE_PACKUMENT: &str = r#"{
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

#[test]
fn explicit_old_bun_add_executes_real_bun_after_policy_pass() {
    let (registry_base_url, server) = serve_packument_once(OLD_PACKAGE_PACKUMENT);
    let temp_dir = temp_test_dir("packvet-fake-bun");
    let fake_bin_dir = temp_dir.join("bin");
    let fake_args_path = temp_dir.join("bun-args.txt");
    write_fake_bun_bin(&fake_bin_dir);

    let output = run_packvet_with_registry_now_and_env(
        &["bun", "add", "old-package"],
        &registry_base_url,
        50 * 60 * 60,
        &[
            ("PATH", path_with_fake_bin(&fake_bin_dir)),
            (
                "PACKVET_FAKE_BUN_ARGS",
                fake_args_path.to_string_lossy().into_owned(),
            ),
        ],
    );

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout is utf-8"),
        "fake bun stdout\n"
    );
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr is utf-8"),
        "\
packvet: checking bun add old-package
packvet: resolving bun metadata for old-package
packvet: skipped review for old-package; older than configured threshold
packvet: running bun add old-package
fake bun stderr
"
    );
    assert_eq!(
        fs::read_to_string(&fake_args_path).expect("fake bun args are captured"),
        "add\nold-package\n"
    );

    let request = server.join().expect("server thread completes");
    assert!(request.starts_with("GET /old-package HTTP/1.1\r\n"));

    fs::remove_dir_all(temp_dir).expect("remove fake bun temp dir");
}

#[test]
fn bun_install_without_package_reads_package_json_and_executes_real_bun_after_policy_pass() {
    let (registry_base_url, server) = serve_packument_once(OLD_PACKAGE_PACKUMENT);
    let temp_dir = temp_test_dir("packvet-bun-package-json");
    fs::create_dir_all(&temp_dir).expect("create bun package temp dir");
    fs::write(
        temp_dir.join("package.json"),
        r#"{"dependencies":{"old-package":"1.1.0"}}"#,
    )
    .expect("write package.json");
    let fake_bin_dir = temp_dir.join("bin");
    let fake_args_path = temp_dir.join("bun-args.txt");
    write_fake_bun_bin(&fake_bin_dir);

    let output = run_packvet_in_dir_with_registry_now_and_env(
        &temp_dir,
        &["bun", "install"],
        &registry_base_url,
        50 * 60 * 60,
        &[
            ("PATH", path_with_fake_bin(&fake_bin_dir)),
            (
                "PACKVET_FAKE_BUN_ARGS",
                fake_args_path.to_string_lossy().into_owned(),
            ),
        ],
    );

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout is utf-8"),
        "fake bun stdout\n"
    );
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr is utf-8"),
        "\
packvet: checking bun install
packvet: resolving bun metadata for old-package@1.1.0
packvet: skipped review for old-package@1.1.0; older than configured threshold
packvet: running bun install
fake bun stderr
"
    );
    assert_eq!(
        fs::read_to_string(&fake_args_path).expect("fake bun args are captured"),
        "install\n"
    );

    let request = server.join().expect("server thread completes");
    assert!(request.starts_with("GET /old-package HTTP/1.1\r\n"));

    fs::remove_dir_all(temp_dir).expect("remove bun package temp dir");
}

#[test]
fn explicit_old_pnpm_add_executes_real_pnpm_after_policy_pass() {
    let (registry_base_url, server) = serve_packument_once(OLD_PACKAGE_PACKUMENT);
    let temp_dir = temp_test_dir("packvet-fake-pnpm");
    let fake_bin_dir = temp_dir.join("bin");
    let fake_args_path = temp_dir.join("pnpm-args.txt");
    write_fake_pnpm_bin(&fake_bin_dir);

    let output = run_packvet_with_registry_now_and_env(
        &["pnpm", "add", "old-package"],
        &registry_base_url,
        50 * 60 * 60,
        &[
            ("PATH", path_with_fake_bin(&fake_bin_dir)),
            (
                "PACKVET_FAKE_PNPM_ARGS",
                fake_args_path.to_string_lossy().into_owned(),
            ),
        ],
    );

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout is utf-8"),
        "fake pnpm stdout\n"
    );
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr is utf-8"),
        "\
packvet: checking pnpm add old-package
packvet: resolving pnpm metadata for old-package
packvet: skipped review for old-package; older than configured threshold
packvet: running pnpm add old-package
fake pnpm stderr
"
    );
    assert_eq!(
        fs::read_to_string(&fake_args_path).expect("fake pnpm args are captured"),
        "add\nold-package\n"
    );

    let request = server.join().expect("server thread completes");
    assert!(request.starts_with("GET /old-package HTTP/1.1\r\n"));

    fs::remove_dir_all(temp_dir).expect("remove fake pnpm temp dir");
}

#[test]
fn pnpm_install_without_package_reads_package_json_and_executes_real_pnpm_after_policy_pass() {
    let (registry_base_url, server) = serve_packument_once(OLD_PACKAGE_PACKUMENT);
    let temp_dir = temp_test_dir("packvet-pnpm-package-json");
    fs::create_dir_all(&temp_dir).expect("create pnpm package temp dir");
    fs::write(
        temp_dir.join("package.json"),
        r#"{"dependencies":{"old-package":"1.1.0"}}"#,
    )
    .expect("write package.json");
    let fake_bin_dir = temp_dir.join("bin");
    let fake_args_path = temp_dir.join("pnpm-args.txt");
    write_fake_pnpm_bin(&fake_bin_dir);

    let output = run_packvet_in_dir_with_registry_now_and_env(
        &temp_dir,
        &["pnpm", "install"],
        &registry_base_url,
        50 * 60 * 60,
        &[
            ("PATH", path_with_fake_bin(&fake_bin_dir)),
            (
                "PACKVET_FAKE_PNPM_ARGS",
                fake_args_path.to_string_lossy().into_owned(),
            ),
        ],
    );

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout is utf-8"),
        "fake pnpm stdout\n"
    );
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr is utf-8"),
        "\
packvet: checking pnpm install
packvet: resolving pnpm metadata for old-package@1.1.0
packvet: skipped review for old-package@1.1.0; older than configured threshold
packvet: running pnpm install
fake pnpm stderr
"
    );
    assert_eq!(
        fs::read_to_string(&fake_args_path).expect("fake pnpm args are captured"),
        "install\n"
    );

    let request = server.join().expect("server thread completes");
    assert!(request.starts_with("GET /old-package HTTP/1.1\r\n"));

    fs::remove_dir_all(temp_dir).expect("remove pnpm package temp dir");
}

#[test]
fn explicit_old_yarn_add_executes_real_yarn_after_policy_pass() {
    let (registry_base_url, server) = serve_packument_once(OLD_PACKAGE_PACKUMENT);
    let temp_dir = temp_test_dir("packvet-fake-yarn");
    let fake_bin_dir = temp_dir.join("bin");
    let fake_args_path = temp_dir.join("yarn-args.txt");
    write_fake_yarn_bin(&fake_bin_dir);

    let output = run_packvet_with_registry_now_and_env(
        &["yarn", "add", "old-package"],
        &registry_base_url,
        50 * 60 * 60,
        &[
            ("PATH", path_with_fake_bin(&fake_bin_dir)),
            (
                "PACKVET_FAKE_YARN_ARGS",
                fake_args_path.to_string_lossy().into_owned(),
            ),
        ],
    );

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout is utf-8"),
        "fake yarn stdout\n"
    );
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr is utf-8"),
        "\
packvet: checking yarn add old-package
packvet: resolving yarn metadata for old-package
packvet: skipped review for old-package; older than configured threshold
packvet: running yarn add old-package
fake yarn stderr
"
    );
    assert_eq!(
        fs::read_to_string(&fake_args_path).expect("fake yarn args are captured"),
        "add\nold-package\n"
    );

    let request = server.join().expect("server thread completes");
    assert!(request.starts_with("GET /old-package HTTP/1.1\r\n"));

    fs::remove_dir_all(temp_dir).expect("remove fake yarn temp dir");
}

#[test]
fn yarn_install_without_package_reads_package_json_and_executes_real_yarn_after_policy_pass() {
    let (registry_base_url, server) = serve_packument_once(OLD_PACKAGE_PACKUMENT);
    let temp_dir = temp_test_dir("packvet-yarn-package-json");
    fs::create_dir_all(&temp_dir).expect("create yarn package temp dir");
    fs::write(
        temp_dir.join("package.json"),
        r#"{"dependencies":{"old-package":"1.1.0"}}"#,
    )
    .expect("write package.json");
    let fake_bin_dir = temp_dir.join("bin");
    let fake_args_path = temp_dir.join("yarn-args.txt");
    write_fake_yarn_bin(&fake_bin_dir);

    let output = run_packvet_in_dir_with_registry_now_and_env(
        &temp_dir,
        &["yarn", "install"],
        &registry_base_url,
        50 * 60 * 60,
        &[
            ("PATH", path_with_fake_bin(&fake_bin_dir)),
            (
                "PACKVET_FAKE_YARN_ARGS",
                fake_args_path.to_string_lossy().into_owned(),
            ),
        ],
    );

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout is utf-8"),
        "fake yarn stdout\n"
    );
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr is utf-8"),
        "\
packvet: checking yarn install
packvet: resolving yarn metadata for old-package@1.1.0
packvet: skipped review for old-package@1.1.0; older than configured threshold
packvet: running yarn install
fake yarn stderr
"
    );
    assert_eq!(
        fs::read_to_string(&fake_args_path).expect("fake yarn args are captured"),
        "install\n"
    );

    let request = server.join().expect("server thread completes");
    assert!(request.starts_with("GET /old-package HTTP/1.1\r\n"));

    fs::remove_dir_all(temp_dir).expect("remove yarn package temp dir");
}
