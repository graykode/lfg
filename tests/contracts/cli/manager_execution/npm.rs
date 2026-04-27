use std::fs;

use crate::cli::support::{
    path_with_fake_bin, run_lfg_with_registry_now_and_env, serve_packument_once, temp_test_dir,
    write_fake_npm_bin,
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
