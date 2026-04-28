use std::fs;

use crate::cli::support::{
    path_with_fake_bin, run_packvet_with_crates_io_registry_now_and_env, serve_packument_once,
    temp_test_dir, write_fake_cargo_bin,
};

#[test]
fn explicit_old_cargo_add_executes_real_cargo_after_policy_pass() {
    let metadata = r#"{
      "crate": { "id": "serde", "max_version": "1.0.1" },
      "versions": [
        {
          "num": "1.0.1",
          "created_at": "1970-01-02T00:00:00+00:00",
          "dl_path": "/api/v1/crates/serde/1.0.1/download"
        },
        {
          "num": "1.0.0",
          "created_at": "1970-01-01T00:00:00+00:00",
          "dl_path": "/api/v1/crates/serde/1.0.0/download"
        }
      ]
    }"#;
    let (registry_base_url, server) = serve_packument_once(metadata);
    let temp_dir = temp_test_dir("packvet-fake-cargo");
    let fake_bin_dir = temp_dir.join("bin");
    let fake_args_path = temp_dir.join("cargo-args.txt");
    write_fake_cargo_bin(&fake_bin_dir);

    let output = run_packvet_with_crates_io_registry_now_and_env(
        &["cargo", "add", "serde"],
        &registry_base_url,
        50 * 60 * 60,
        &[
            ("PATH", path_with_fake_bin(&fake_bin_dir)),
            (
                "PACKVET_FAKE_CARGO_ARGS",
                fake_args_path.to_string_lossy().into_owned(),
            ),
        ],
    );

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout is utf-8"),
        "fake cargo stdout\n"
    );
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr is utf-8"),
        "fake cargo stderr\n"
    );
    assert_eq!(
        fs::read_to_string(&fake_args_path).expect("fake cargo args are captured"),
        "add\nserde\n"
    );

    let request = server.join().expect("server thread completes");
    assert!(request.starts_with("GET /api/v1/crates/serde HTTP/1.1\r\n"));

    fs::remove_dir_all(temp_dir).expect("remove fake cargo temp dir");
}
