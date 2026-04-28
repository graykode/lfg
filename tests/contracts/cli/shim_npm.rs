use std::fs;

use super::support::{
    path_with_bin_dirs, run_program_with_registry_now_and_env, serve_recent_package_with_archives,
    temp_test_dir, write_fake_npm_bin, write_packvet_shim,
};

#[test]
fn shim_npm_install_detects_manager_from_argv0_and_skips_shim_for_real_npm() {
    let (registry_base_url, server) = serve_recent_package_with_archives();
    let temp_dir = temp_test_dir("packvet-shim-npm");
    let shim_bin_dir = temp_dir.join("shim-bin");
    let real_bin_dir = temp_dir.join("real-bin");
    let fake_args_path = temp_dir.join("npm-args.txt");
    let shim_path = write_packvet_shim(&shim_bin_dir, "npm");
    write_fake_npm_bin(&real_bin_dir);

    let output = run_program_with_registry_now_and_env(
        &shim_path,
        &["install", "recent-package"],
        &registry_base_url,
        50 * 60 * 60,
        &[
            ("PATH", path_with_bin_dirs(&[&shim_bin_dir, &real_bin_dir])),
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
        "fake npm stderr\n"
    );
    assert_eq!(
        fs::read_to_string(&fake_args_path).expect("fake npm args are captured"),
        "install\nrecent-package\n"
    );

    let requests = server.join().expect("server thread completes");
    assert_eq!(requests.len(), 1);
    assert!(requests[0].starts_with("GET /recent-package HTTP/1.1\r\n"));

    fs::remove_dir_all(temp_dir).expect("remove shim temp dir");
}

#[test]
fn shim_npm_i_alias_invokes_guard_and_real_npm() {
    let (registry_base_url, server) = serve_recent_package_with_archives();
    let temp_dir = temp_test_dir("packvet-shim-npm-i");
    let shim_bin_dir = temp_dir.join("shim-bin");
    let real_bin_dir = temp_dir.join("real-bin");
    let fake_args_path = temp_dir.join("npm-args.txt");
    let shim_path = write_packvet_shim(&shim_bin_dir, "npm");
    write_fake_npm_bin(&real_bin_dir);

    let output = run_program_with_registry_now_and_env(
        &shim_path,
        &["i", "recent-package"],
        &registry_base_url,
        50 * 60 * 60,
        &[
            ("PATH", path_with_bin_dirs(&[&shim_bin_dir, &real_bin_dir])),
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
        fs::read_to_string(&fake_args_path).expect("fake npm args are captured"),
        "i\nrecent-package\n"
    );

    let requests = server.join().expect("server thread completes");
    assert_eq!(requests.len(), 1);
    assert!(requests[0].starts_with("GET /recent-package HTTP/1.1\r\n"));

    fs::remove_dir_all(temp_dir).expect("remove shim npm i temp dir");
}

#[test]
fn shim_npm_install_can_bypass_review_for_emergency_use() {
    let temp_dir = temp_test_dir("packvet-shim-bypass");
    let shim_bin_dir = temp_dir.join("shim-bin");
    let real_bin_dir = temp_dir.join("real-bin");
    let fake_args_path = temp_dir.join("npm-args.txt");
    let shim_path = write_packvet_shim(&shim_bin_dir, "npm");
    write_fake_npm_bin(&real_bin_dir);

    let output = run_program_with_registry_now_and_env(
        &shim_path,
        &["install", "bypass-package"],
        "http://127.0.0.1:9",
        50 * 60 * 60,
        &[
            ("PATH", path_with_bin_dirs(&[&shim_bin_dir, &real_bin_dir])),
            ("PACKVET_BYPASS", "1".to_owned()),
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
        "fake npm stderr\n"
    );
    assert_eq!(
        fs::read_to_string(&fake_args_path).expect("fake npm args are captured"),
        "install\nbypass-package\n"
    );

    fs::remove_dir_all(temp_dir).expect("remove bypass temp dir");
}

#[test]
fn shim_npm_install_returns_ask_when_real_npm_is_missing() {
    let (registry_base_url, server) = serve_recent_package_with_archives();
    let temp_dir = temp_test_dir("packvet-shim-missing-real-npm");
    let shim_bin_dir = temp_dir.join("shim-bin");
    let shim_path = write_packvet_shim(&shim_bin_dir, "npm");

    let output = run_program_with_registry_now_and_env(
        &shim_path,
        &["install", "recent-package"],
        &registry_base_url,
        50 * 60 * 60,
        &[(
            "PATH",
            std::env::join_paths([&shim_bin_dir])
                .expect("join isolated PATH")
                .to_string_lossy()
                .into_owned(),
        )],
    );

    assert_eq!(output.status.code(), Some(20));
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr is utf-8"),
        "packvet: npm executable is unavailable; install is paused.\n"
    );

    let requests = server.join().expect("server thread completes");
    assert_eq!(requests.len(), 1);
    assert!(requests[0].starts_with("GET /recent-package HTTP/1.1\r\n"));

    fs::remove_dir_all(temp_dir).expect("remove missing-real-npm temp dir");
}
