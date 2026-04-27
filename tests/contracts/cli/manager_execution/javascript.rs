use std::fs;

use crate::cli::support::{
    path_with_fake_bin, run_lfg_with_registry_now_and_env, serve_packument_once, temp_test_dir,
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
fn explicit_old_pnpm_add_executes_real_pnpm_after_policy_pass() {
    let (registry_base_url, server) = serve_packument_once(OLD_PACKAGE_PACKUMENT);
    let temp_dir = temp_test_dir("lfg-fake-pnpm");
    let fake_bin_dir = temp_dir.join("bin");
    let fake_args_path = temp_dir.join("pnpm-args.txt");
    write_fake_pnpm_bin(&fake_bin_dir);

    let output = run_lfg_with_registry_now_and_env(
        &["pnpm", "add", "old-package"],
        &registry_base_url,
        50 * 60 * 60,
        &[
            ("PATH", path_with_fake_bin(&fake_bin_dir)),
            (
                "LFG_FAKE_PNPM_ARGS",
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
        "fake pnpm stderr\n"
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
fn explicit_old_yarn_add_executes_real_yarn_after_policy_pass() {
    let (registry_base_url, server) = serve_packument_once(OLD_PACKAGE_PACKUMENT);
    let temp_dir = temp_test_dir("lfg-fake-yarn");
    let fake_bin_dir = temp_dir.join("bin");
    let fake_args_path = temp_dir.join("yarn-args.txt");
    write_fake_yarn_bin(&fake_bin_dir);

    let output = run_lfg_with_registry_now_and_env(
        &["yarn", "add", "old-package"],
        &registry_base_url,
        50 * 60 * 60,
        &[
            ("PATH", path_with_fake_bin(&fake_bin_dir)),
            (
                "LFG_FAKE_YARN_ARGS",
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
        "fake yarn stderr\n"
    );
    assert_eq!(
        fs::read_to_string(&fake_args_path).expect("fake yarn args are captured"),
        "add\nold-package\n"
    );

    let request = server.join().expect("server thread completes");
    assert!(request.starts_with("GET /old-package HTTP/1.1\r\n"));

    fs::remove_dir_all(temp_dir).expect("remove fake yarn temp dir");
}
