use std::fs;

use super::support::{run_packvet, temp_test_dir};

#[test]
fn shim_install_and_uninstall_are_reversible() {
    let temp_dir = temp_test_dir("packvet-shim-setup");
    let shim_dir = temp_dir.join("bin");

    let install_output = run_packvet(&[
        "shim",
        "install",
        "--dir",
        &shim_dir.to_string_lossy(),
        "npm",
    ]);

    assert_eq!(install_output.status.code(), Some(0));
    assert!(install_output.stderr.is_empty());
    assert_eq!(
        String::from_utf8(install_output.stdout).expect("stdout is utf-8"),
        format!(
            "packvet: installed npm shim at {}\n",
            shim_dir.join("npm").display()
        )
    );
    assert_eq!(
        fs::canonicalize(shim_dir.join("npm")).expect("shim target canonicalizes"),
        fs::canonicalize(env!("CARGO_BIN_EXE_packvet")).expect("packvet binary canonicalizes")
    );

    let uninstall_output = run_packvet(&[
        "shim",
        "uninstall",
        "--dir",
        &shim_dir.to_string_lossy(),
        "npm",
    ]);

    assert_eq!(uninstall_output.status.code(), Some(0));
    assert!(uninstall_output.stderr.is_empty());
    assert_eq!(
        String::from_utf8(uninstall_output.stdout).expect("stdout is utf-8"),
        format!(
            "packvet: removed npm shim from {}\n",
            shim_dir.join("npm").display()
        )
    );
    assert!(!shim_dir.join("npm").exists());

    fs::remove_dir_all(temp_dir).expect("remove shim setup temp dir");
}

#[test]
fn shim_install_refuses_to_replace_existing_file() {
    let temp_dir = temp_test_dir("packvet-shim-existing-file");
    let shim_dir = temp_dir.join("bin");
    fs::create_dir_all(&shim_dir).expect("create shim dir");
    fs::write(shim_dir.join("npm"), "not managed by packvet").expect("write existing file");

    let output = run_packvet(&[
        "shim",
        "install",
        "--dir",
        &shim_dir.to_string_lossy(),
        "npm",
    ]);

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr is utf-8"),
        format!(
            "packvet: shim target already exists: {}\n",
            shim_dir.join("npm").display()
        )
    );
    assert_eq!(
        fs::read_to_string(shim_dir.join("npm")).expect("existing file remains"),
        "not managed by packvet"
    );

    fs::remove_dir_all(temp_dir).expect("remove existing file temp dir");
}
