use std::fs;

use crate::cli::manager_execution::python::OLD_PYTHON_PROJECT;
use crate::cli::support::{
    path_with_fake_bin, run_packvet_with_pypi_registry_now_and_env, serve_packument_once,
    temp_test_dir, write_fake_uv_bin,
};

#[test]
fn explicit_old_uv_add_executes_real_uv_after_policy_pass() {
    let (registry_base_url, server) = serve_packument_once(OLD_PYTHON_PROJECT);
    let temp_dir = temp_test_dir("packvet-fake-uv");
    let fake_bin_dir = temp_dir.join("bin");
    let fake_args_path = temp_dir.join("uv-args.txt");
    write_fake_uv_bin(&fake_bin_dir);

    let output = run_packvet_with_pypi_registry_now_and_env(
        &["uv", "add", "old-python-package"],
        &registry_base_url,
        50 * 60 * 60,
        &[
            ("PATH", path_with_fake_bin(&fake_bin_dir)),
            (
                "PACKVET_FAKE_UV_ARGS",
                fake_args_path.to_string_lossy().into_owned(),
            ),
        ],
    );

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout is utf-8"),
        "fake uv stdout\n"
    );
    let stderr = String::from_utf8(output.stderr).expect("stderr is utf-8");
    assert_eq!(
        stderr,
        "\
packvet: checking uv add old-python-package
packvet: resolving uv metadata for old-python-package
packvet: skipped review for old-python-package; older than configured threshold
packvet: running uv add old-python-package
fake uv stderr
"
    );
    assert_eq!(
        fs::read_to_string(&fake_args_path).expect("fake uv args are captured"),
        "add\nold-python-package\n"
    );

    let request = server.join().expect("server thread completes");
    assert!(request.starts_with("GET /pypi/old-python-package/json HTTP/1.1\r\n"));

    fs::remove_dir_all(temp_dir).expect("remove fake uv temp dir");
}
