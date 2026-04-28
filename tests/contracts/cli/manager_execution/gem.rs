use std::fs;

use crate::cli::support::{
    path_with_fake_bin, run_packvet_with_rubygems_registry_now_and_env, serve_packument_once,
    temp_test_dir, write_fake_gem_bin,
};

#[test]
fn explicit_old_gem_install_executes_real_gem_after_policy_pass() {
    let versions = r#"[
      {
        "number": "3.0.0",
        "created_at": "1970-01-02T00:00:00.000Z"
      },
      {
        "number": "2.2.0",
        "created_at": "1970-01-01T00:00:00.000Z"
      }
    ]"#;
    let (registry_base_url, server) = serve_packument_once(versions);
    let temp_dir = temp_test_dir("packvet-fake-gem");
    let fake_bin_dir = temp_dir.join("bin");
    let fake_args_path = temp_dir.join("gem-args.txt");
    write_fake_gem_bin(&fake_bin_dir);

    let output = run_packvet_with_rubygems_registry_now_and_env(
        &["gem", "install", "rack"],
        &registry_base_url,
        50 * 60 * 60,
        &[
            ("PATH", path_with_fake_bin(&fake_bin_dir)),
            (
                "PACKVET_FAKE_GEM_ARGS",
                fake_args_path.to_string_lossy().into_owned(),
            ),
        ],
    );

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout is utf-8"),
        "fake gem stdout\n"
    );
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr is utf-8"),
        "fake gem stderr\n"
    );
    assert_eq!(
        fs::read_to_string(&fake_args_path).expect("fake gem args are captured"),
        "install\nrack\n"
    );

    let request = server.join().expect("server thread completes");
    assert!(request.starts_with("GET /api/v1/versions/rack.json HTTP/1.1\r\n"));

    fs::remove_dir_all(temp_dir).expect("remove fake gem temp dir");
}
