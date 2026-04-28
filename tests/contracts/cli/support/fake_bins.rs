use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

pub(crate) fn write_fake_npm_bin(dir: &Path) {
    write_fake_manager_bin(
        dir,
        "npm",
        "PACKVET_FAKE_NPM_ARGS",
        "fake npm stdout",
        "fake npm stderr",
    );
}

pub(crate) fn write_fake_bun_bin(dir: &Path) {
    write_fake_manager_bin(
        dir,
        "bun",
        "PACKVET_FAKE_BUN_ARGS",
        "fake bun stdout",
        "fake bun stderr",
    );
}

pub(crate) fn write_fake_cargo_bin(dir: &Path) {
    write_fake_manager_bin(
        dir,
        "cargo",
        "PACKVET_FAKE_CARGO_ARGS",
        "fake cargo stdout",
        "fake cargo stderr",
    );
}

pub(crate) fn write_fake_gem_bin(dir: &Path) {
    write_fake_manager_bin(
        dir,
        "gem",
        "PACKVET_FAKE_GEM_ARGS",
        "fake gem stdout",
        "fake gem stderr",
    );
}

pub(crate) fn write_fake_pip_bin(dir: &Path) {
    write_fake_manager_bin(
        dir,
        "pip",
        "PACKVET_FAKE_PIP_ARGS",
        "fake pip stdout",
        "fake pip stderr",
    );
}

pub(crate) fn write_fake_pnpm_bin(dir: &Path) {
    write_fake_manager_bin(
        dir,
        "pnpm",
        "PACKVET_FAKE_PNPM_ARGS",
        "fake pnpm stdout",
        "fake pnpm stderr",
    );
}

pub(crate) fn write_fake_uv_bin(dir: &Path) {
    write_fake_manager_bin(
        dir,
        "uv",
        "PACKVET_FAKE_UV_ARGS",
        "fake uv stdout",
        "fake uv stderr",
    );
}

pub(crate) fn write_fake_yarn_bin(dir: &Path) {
    write_fake_manager_bin(
        dir,
        "yarn",
        "PACKVET_FAKE_YARN_ARGS",
        "fake yarn stdout",
        "fake yarn stderr",
    );
}

fn write_fake_manager_bin(dir: &Path, manager: &str, args_env: &str, stdout: &str, stderr: &str) {
    fs::create_dir_all(dir).expect("create fake bin dir");
    let manager_path = dir.join(manager);
    fs::write(
        &manager_path,
        format!(
            "#!/bin/sh\nprintf '%s\\n' \"$@\" > \"${args_env}\"\nprintf '{stdout}\\n'\nprintf '{stderr}\\n' >&2\n",
        ),
    )
    .expect("write fake manager");
    mark_executable(&manager_path);
}

pub(crate) fn write_fake_claude_bin(dir: &Path, provider_output: &str) {
    fs::create_dir_all(dir).expect("create fake bin dir");
    let claude_path = dir.join("claude");
    fs::write(
        &claude_path,
        format!(
            "#!/bin/sh\ncat > \"$PACKVET_FAKE_PROVIDER_PROMPT\"\ncat <<'PACKVET_PROVIDER_OUTPUT'\n{provider_output}PACKVET_PROVIDER_OUTPUT\n"
        ),
    )
    .expect("write fake claude");
    mark_executable(&claude_path);
}

fn mark_executable(path: &Path) {
    let mut permissions = fs::metadata(path)
        .expect("read executable metadata")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).expect("mark file executable");
}

pub(crate) fn path_with_fake_bin(bin_dir: &Path) -> String {
    path_with_bin_dirs(&[bin_dir])
}

pub(crate) fn path_with_bin_dirs(bin_dirs: &[&Path]) -> String {
    let existing = std::env::var_os("PATH").unwrap_or_default();
    let paths = bin_dirs
        .iter()
        .map(|path| (*path).to_path_buf())
        .chain(std::env::split_paths(&existing));
    std::env::join_paths(paths)
        .expect("join PATH")
        .to_string_lossy()
        .into_owned()
}
