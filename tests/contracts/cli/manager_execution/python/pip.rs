use std::collections::BTreeMap;
use std::fs;

use crate::cli::manager_execution::python::OLD_PYTHON_PROJECT;
use crate::cli::support::{
    path_with_fake_bin, run_lfg_with_pypi_registry_now_and_env, serve_json_paths_once,
    serve_packument_once, temp_test_dir, write_fake_pip_bin,
};

#[test]
fn explicit_old_pip_install_executes_real_pip_after_policy_pass() {
    let (registry_base_url, server) = serve_packument_once(OLD_PYTHON_PROJECT);
    let temp_dir = temp_test_dir("lfg-fake-pip");
    let fake_bin_dir = temp_dir.join("bin");
    let fake_args_path = temp_dir.join("pip-args.txt");
    write_fake_pip_bin(&fake_bin_dir);

    let output = run_lfg_with_pypi_registry_now_and_env(
        &["pip", "install", "old-python-package"],
        &registry_base_url,
        50 * 60 * 60,
        &[
            ("PATH", path_with_fake_bin(&fake_bin_dir)),
            (
                "LFG_FAKE_PIP_ARGS",
                fake_args_path.to_string_lossy().into_owned(),
            ),
        ],
    );

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout is utf-8"),
        "fake pip stdout\n"
    );
    let stderr = String::from_utf8(output.stderr).expect("stderr is utf-8");
    assert_eq!(stderr, "fake pip stderr\n");
    assert_eq!(
        fs::read_to_string(&fake_args_path).expect("fake pip args are captured"),
        "install\nold-python-package\n"
    );

    let request = server.join().expect("server thread completes");
    assert!(request.starts_with("GET /pypi/old-python-package/json HTTP/1.1\r\n"));

    fs::remove_dir_all(temp_dir).expect("remove fake pip temp dir");
}

#[test]
fn explicit_old_pip_requirements_install_executes_real_pip_after_policy_pass() {
    let first_project = r#"{
      "info": { "name": "first-python-package", "version": "1.1.0" },
      "releases": {
        "1.0.0": [
          {
            "packagetype": "sdist",
            "url": "https://files.pythonhosted.org/packages/first-python-package-1.0.0.tar.gz",
            "upload_time_iso_8601": "1970-01-01T00:00:00.000000Z"
          }
        ],
        "1.1.0": [
          {
            "packagetype": "sdist",
            "url": "https://files.pythonhosted.org/packages/first-python-package-1.1.0.tar.gz",
            "upload_time_iso_8601": "1970-01-02T00:00:00.000000Z"
          }
        ]
      }
    }"#;
    let second_project = r#"{
      "info": { "name": "second-python-package", "version": "2.0.0" },
      "releases": {
        "1.0.0": [
          {
            "packagetype": "sdist",
            "url": "https://files.pythonhosted.org/packages/second-python-package-1.0.0.tar.gz",
            "upload_time_iso_8601": "1970-01-01T00:00:00.000000Z"
          }
        ],
        "2.0.0": [
          {
            "packagetype": "sdist",
            "url": "https://files.pythonhosted.org/packages/second-python-package-2.0.0.tar.gz",
            "upload_time_iso_8601": "1970-01-02T00:00:00.000000Z"
          }
        ]
      }
    }"#;
    let (registry_base_url, server) = serve_json_paths_once(BTreeMap::from([
        (
            "/pypi/first-python-package/json".to_owned(),
            first_project.to_owned(),
        ),
        (
            "/pypi/second-python-package/json".to_owned(),
            second_project.to_owned(),
        ),
    ]));
    let temp_dir = temp_test_dir("lfg-fake-pip-requirements");
    let fake_bin_dir = temp_dir.join("bin");
    let fake_args_path = temp_dir.join("pip-args.txt");
    let requirements_path = temp_dir.join("requirements.txt");
    fs::create_dir_all(&temp_dir).expect("create temp dir");
    fs::write(
        &requirements_path,
        "first-python-package\nsecond-python-package==2.0.0\n",
    )
    .expect("write requirements file");
    write_fake_pip_bin(&fake_bin_dir);

    let output = run_lfg_with_pypi_registry_now_and_env(
        &["pip", "install", "-r", &requirements_path.to_string_lossy()],
        &registry_base_url,
        50 * 60 * 60,
        &[
            ("PATH", path_with_fake_bin(&fake_bin_dir)),
            (
                "LFG_FAKE_PIP_ARGS",
                fake_args_path.to_string_lossy().into_owned(),
            ),
        ],
    );

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        String::from_utf8(output.stdout).expect("stdout is utf-8"),
        "fake pip stdout\n"
    );
    assert_eq!(
        fs::read_to_string(&fake_args_path).expect("fake pip args are captured"),
        format!("install\n-r\n{}\n", requirements_path.display())
    );

    let requests = server.join().expect("server thread completes");
    assert_eq!(requests.len(), 2);
    assert!(requests
        .iter()
        .any(|request| request.starts_with("GET /pypi/first-python-package/json HTTP/1.1\r\n")));
    assert!(requests
        .iter()
        .any(|request| request.starts_with("GET /pypi/second-python-package/json HTTP/1.1\r\n")));

    fs::remove_dir_all(temp_dir).expect("remove fake pip requirements temp dir");
}
