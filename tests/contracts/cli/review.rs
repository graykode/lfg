use std::fs;

use super::support::{
    path_with_fake_bin, run_packvet_with_crates_io_registry_now_and_env,
    run_packvet_with_pypi_registry_now_and_env, run_packvet_with_registry_and_now,
    run_packvet_with_registry_now_and_env, run_packvet_with_rubygems_registry_now_and_env,
    serve_recent_crate_with_archives, serve_recent_gem_with_archives,
    serve_recent_package_with_archives, serve_recent_python_project_with_archives, temp_test_dir,
    write_fake_claude_bin, write_fake_npm_bin,
};

#[test]
fn explicit_recent_npm_install_fetches_metadata_and_pauses_for_diff_review() {
    let (registry_base_url, server) = serve_recent_package_with_archives();

    let output = run_packvet_with_registry_and_now(
        &["npm", "install", "recent-package"],
        &registry_base_url,
        25 * 60 * 60,
    );

    assert_recent_review_pause(output, "npm", "install");

    let requests = server.join().expect("server thread completes");
    assert_recent_npm_archive_requests(&requests);
}

#[test]
fn explicit_recent_pnpm_add_fetches_metadata_and_pauses_for_diff_review() {
    let (registry_base_url, server) = serve_recent_package_with_archives();

    let output = run_packvet_with_registry_and_now(
        &["pnpm", "add", "recent-package"],
        &registry_base_url,
        25 * 60 * 60,
    );

    assert_recent_review_pause(output, "pnpm", "add");

    let requests = server.join().expect("server thread completes");
    assert_recent_npm_archive_requests(&requests);
}

#[test]
fn explicit_recent_yarn_add_fetches_metadata_and_pauses_for_diff_review() {
    let (registry_base_url, server) = serve_recent_package_with_archives();

    let output = run_packvet_with_registry_and_now(
        &["yarn", "add", "recent-package"],
        &registry_base_url,
        25 * 60 * 60,
    );

    assert_recent_review_pause(output, "yarn", "add");

    let requests = server.join().expect("server thread completes");
    assert_recent_npm_archive_requests(&requests);
}

#[test]
fn explicit_recent_pip_install_fetches_metadata_and_pauses_for_diff_review() {
    let (registry_base_url, server) = serve_recent_python_project_with_archives();

    let output = run_packvet_with_pypi_registry_now_and_env(
        &["pip", "install", "recent-python-package"],
        &registry_base_url,
        25 * 60 * 60,
        &[],
    );

    assert_recent_review_pause(output, "pip", "install");

    let requests = server.join().expect("server thread completes");
    assert_eq!(requests.len(), 3);
    assert!(requests[0].starts_with("GET /pypi/recent-python-package/json HTTP/1.1\r\n"));
    assert!(requests.iter().any(|request| {
        request.starts_with("GET /recent-python-package-1.0.0.tar.gz HTTP/1.1\r\n")
    }));
    assert!(requests.iter().any(|request| {
        request.starts_with("GET /recent-python-package-1.1.0.tar.gz HTTP/1.1\r\n")
    }));
}

#[test]
fn explicit_recent_uv_add_fetches_metadata_and_pauses_for_diff_review() {
    let (registry_base_url, server) = serve_recent_python_project_with_archives();

    let output = run_packvet_with_pypi_registry_now_and_env(
        &["uv", "add", "recent-python-package"],
        &registry_base_url,
        25 * 60 * 60,
        &[],
    );

    assert_recent_review_pause(output, "uv", "add");

    let requests = server.join().expect("server thread completes");
    assert_eq!(requests.len(), 3);
    assert!(requests[0].starts_with("GET /pypi/recent-python-package/json HTTP/1.1\r\n"));
    assert!(requests.iter().any(|request| {
        request.starts_with("GET /recent-python-package-1.0.0.tar.gz HTTP/1.1\r\n")
    }));
    assert!(requests.iter().any(|request| {
        request.starts_with("GET /recent-python-package-1.1.0.tar.gz HTTP/1.1\r\n")
    }));
}

#[test]
fn explicit_recent_cargo_add_fetches_metadata_and_pauses_for_diff_review() {
    let (registry_base_url, server) = serve_recent_crate_with_archives();

    let output = run_packvet_with_crates_io_registry_now_and_env(
        &["cargo", "add", "recent-crate"],
        &registry_base_url,
        25 * 60 * 60,
        &[],
    );

    assert_recent_review_pause(output, "cargo", "add");

    let requests = server.join().expect("server thread completes");
    assert_eq!(requests.len(), 3);
    assert!(requests[0].starts_with("GET /api/v1/crates/recent-crate HTTP/1.1\r\n"));
    assert!(requests.iter().any(|request| request
        .starts_with("GET /api/v1/crates/recent-crate/1.0.0/download HTTP/1.1\r\n")));
    assert!(requests.iter().any(|request| request
        .starts_with("GET /api/v1/crates/recent-crate/1.1.0/download HTTP/1.1\r\n")));
}

#[test]
fn explicit_recent_gem_install_fetches_metadata_and_pauses_for_diff_review() {
    let (registry_base_url, server) = serve_recent_gem_with_archives();

    let output = run_packvet_with_rubygems_registry_now_and_env(
        &["gem", "install", "recent-gem"],
        &registry_base_url,
        25 * 60 * 60,
        &[],
    );

    assert_recent_review_pause(output, "gem", "install");

    let requests = server.join().expect("server thread completes");
    assert_eq!(requests.len(), 3);
    assert!(requests[0].starts_with("GET /api/v1/versions/recent-gem.json HTTP/1.1\r\n"));
    assert!(requests
        .iter()
        .any(|request| request.starts_with("GET /gems/recent-gem-1.0.0.gem HTTP/1.1\r\n")));
    assert!(requests
        .iter()
        .any(|request| request.starts_with("GET /gems/recent-gem-1.1.0.gem HTTP/1.1\r\n")));
}

#[test]
fn explicit_recent_npm_install_logs_review_and_executes_real_npm_after_provider_pass() {
    let (registry_base_url, server) = serve_recent_package_with_archives();
    let temp_dir = temp_test_dir("packvet-fake-provider-pass");
    let fake_bin_dir = temp_dir.join("bin");
    let fake_args_path = temp_dir.join("npm-args.txt");
    let fake_prompt_path = temp_dir.join("provider-prompt.txt");
    let review_log_dir = temp_dir.join("reviews");
    write_fake_npm_bin(&fake_bin_dir);
    write_fake_claude_bin(
        &fake_bin_dir,
        "verdict: pass\nreason: fixture allowed\n\nevidence:\n- package/index.js: fixture signal\n",
    );

    let output = run_packvet_with_registry_now_and_env(
        &["npm", "install", "recent-package"],
        &registry_base_url,
        25 * 60 * 60,
        &[
            ("PATH", path_with_fake_bin(&fake_bin_dir)),
            ("PACKVET_REVIEW_PROVIDER", "claude".to_owned()),
            (
                "PACKVET_FAKE_NPM_ARGS",
                fake_args_path.to_string_lossy().into_owned(),
            ),
            (
                "PACKVET_FAKE_PROVIDER_PROMPT",
                fake_prompt_path.to_string_lossy().into_owned(),
            ),
            (
                "PACKVET_REVIEW_LOG_DIR",
                review_log_dir.to_string_lossy().into_owned(),
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
    let review_log =
        fs::read_to_string(review_log_dir.join("reviews.jsonl")).expect("review log is written");
    assert!(review_log.contains("\"package\":\"recent-package\""));
    assert!(review_log.contains("\"provider_id\":\"claude-cli\""));
    assert!(review_log.contains("\"provider_output\":\"verdict: pass"));
    assert!(review_log.contains("\"verdict\":\"pass\""));
    assert!(review_log.contains("\"prompt\":\"You are reviewing a package source diff"));

    let requests = server.join().expect("server thread completes");
    assert_eq!(requests.len(), 3);

    fs::remove_dir_all(temp_dir).expect("remove fake provider temp dir");
}

#[test]
fn explicit_recent_npm_install_does_not_execute_real_npm_after_provider_block() {
    let (registry_base_url, server) = serve_recent_package_with_archives();
    let temp_dir = temp_test_dir("packvet-fake-provider-block");
    let fake_bin_dir = temp_dir.join("bin");
    let fake_args_path = temp_dir.join("npm-args.txt");
    let fake_prompt_path = temp_dir.join("provider-prompt.txt");
    let review_log_dir = temp_dir.join("reviews");
    write_fake_npm_bin(&fake_bin_dir);
    write_fake_claude_bin(
        &fake_bin_dir,
        "verdict: block\nreason: fixture blocked\n\nevidence:\n- package/index.js: fixture signal\n",
    );

    let output = run_packvet_with_registry_now_and_env(
        &["npm", "install", "recent-package"],
        &registry_base_url,
        25 * 60 * 60,
        &[
            ("PATH", path_with_fake_bin(&fake_bin_dir)),
            ("PACKVET_REVIEW_PROVIDER", "claude".to_owned()),
            (
                "PACKVET_FAKE_NPM_ARGS",
                fake_args_path.to_string_lossy().into_owned(),
            ),
            (
                "PACKVET_FAKE_PROVIDER_PROMPT",
                fake_prompt_path.to_string_lossy().into_owned(),
            ),
            (
                "PACKVET_REVIEW_LOG_DIR",
                review_log_dir.to_string_lossy().into_owned(),
            ),
        ],
    );

    assert_eq!(output.status.code(), Some(30));
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr is utf-8"),
        "packvet: npm install was blocked by provider review.\n"
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
fn explicit_recent_npm_install_can_print_review_prompt_for_debugging() {
    let (registry_base_url, server) = serve_recent_package_with_archives();
    let temp_dir = temp_test_dir("packvet-print-provider-prompt");
    let fake_bin_dir = temp_dir.join("bin");
    let fake_args_path = temp_dir.join("npm-args.txt");
    let fake_prompt_path = temp_dir.join("provider-prompt.txt");
    let review_log_dir = temp_dir.join("reviews");
    write_fake_npm_bin(&fake_bin_dir);
    write_fake_claude_bin(
        &fake_bin_dir,
        "verdict: block\nreason: fixture blocked\n\nevidence:\n- package/index.js: fixture signal\n",
    );

    let output = run_packvet_with_registry_now_and_env(
        &["npm", "install", "recent-package"],
        &registry_base_url,
        25 * 60 * 60,
        &[
            ("PATH", path_with_fake_bin(&fake_bin_dir)),
            ("PACKVET_REVIEW_PROVIDER", "claude".to_owned()),
            ("PACKVET_PRINT_REVIEW_PROMPT", "1".to_owned()),
            (
                "PACKVET_FAKE_NPM_ARGS",
                fake_args_path.to_string_lossy().into_owned(),
            ),
            (
                "PACKVET_FAKE_PROVIDER_PROMPT",
                fake_prompt_path.to_string_lossy().into_owned(),
            ),
            (
                "PACKVET_REVIEW_LOG_DIR",
                review_log_dir.to_string_lossy().into_owned(),
            ),
        ],
    );

    assert_eq!(output.status.code(), Some(30));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).expect("stderr is utf-8");
    assert!(stderr.contains("----- packvet review prompt -----\n"));
    assert!(stderr.contains("package: recent-package"));
    assert!(stderr.contains("previous version: 1.0.0"));
    assert!(stderr.contains("target version: 1.1.0"));
    assert!(stderr.contains("+module.exports = 2;"));
    assert!(stderr.contains("----- end packvet review prompt -----\n"));
    assert!(stderr.ends_with("packvet: npm install was blocked by provider review.\n"));
    assert!(!fake_args_path.exists());

    let requests = server.join().expect("server thread completes");
    assert_eq!(requests.len(), 3);

    fs::remove_dir_all(temp_dir).expect("remove fake provider temp dir");
}

#[test]
fn explicit_npm_install_uses_configured_review_age_threshold() {
    let (registry_base_url, server) = serve_recent_package_with_archives();
    let temp_dir = temp_test_dir("packvet-fake-threshold");
    let fake_bin_dir = temp_dir.join("bin");
    let fake_args_path = temp_dir.join("npm-args.txt");
    write_fake_npm_bin(&fake_bin_dir);

    let output = run_packvet_with_registry_now_and_env(
        &["npm", "install", "recent-package"],
        &registry_base_url,
        50 * 60 * 60,
        &[
            ("PATH", path_with_fake_bin(&fake_bin_dir)),
            (
                "PACKVET_FAKE_NPM_ARGS",
                fake_args_path.to_string_lossy().into_owned(),
            ),
            (
                "PACKVET_REVIEW_AGE_THRESHOLD_SECONDS",
                (48 * 60 * 60).to_string(),
            ),
        ],
    );

    assert_eq!(output.status.code(), Some(20));
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr is utf-8"),
        "packvet: review required for npm install, but provider review is not wired yet. install is paused.\n"
    );
    assert!(!fake_args_path.exists());

    let requests = server.join().expect("server thread completes");
    assert_eq!(requests.len(), 3);

    fs::remove_dir_all(temp_dir).expect("remove fake threshold temp dir");
}

fn assert_recent_review_pause(output: std::process::Output, manager: &str, operation: &str) {
    assert_eq!(output.status.code(), Some(20));
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr is utf-8"),
        format!(
            "packvet: review required for {manager} {operation}, but provider review is not wired yet. install is paused.\n"
        )
    );
}

fn assert_recent_npm_archive_requests(requests: &[String]) {
    assert_eq!(requests.len(), 3);
    assert!(requests[0].starts_with("GET /recent-package HTTP/1.1\r\n"));
    assert!(requests
        .iter()
        .any(|request| request.starts_with("GET /recent-package-1.0.0.tgz HTTP/1.1\r\n")));
    assert!(requests
        .iter()
        .any(|request| request.starts_with("GET /recent-package-1.1.0.tgz HTTP/1.1\r\n")));
}
