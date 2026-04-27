use std::path::Path;
use std::process::Command;

pub(crate) fn run_lfg(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_lfg"))
        .args(args)
        .output()
        .expect("run lfg binary")
}

pub(crate) fn run_lfg_with_registry_and_now(
    args: &[&str],
    registry_base_url: &str,
    now_unix_seconds: u64,
) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_lfg"))
        .args(args)
        .env("LFG_NPM_REGISTRY_URL", registry_base_url)
        .env("LFG_NOW_UNIX_SECONDS", now_unix_seconds.to_string())
        .env("LFG_REVIEW_PROVIDER", "none")
        .output()
        .expect("run lfg binary")
}

pub(crate) fn run_lfg_with_registry_now_and_env(
    args: &[&str],
    registry_base_url: &str,
    now_unix_seconds: u64,
    envs: &[(&str, String)],
) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_lfg"));
    command
        .args(args)
        .env("LFG_NPM_REGISTRY_URL", registry_base_url)
        .env("LFG_NOW_UNIX_SECONDS", now_unix_seconds.to_string())
        .env("LFG_REVIEW_PROVIDER", "none");

    for (key, value) in envs {
        command.env(key, value);
    }

    command.output().expect("run lfg binary")
}

pub(crate) fn run_lfg_with_pypi_registry_now_and_env(
    args: &[&str],
    registry_base_url: &str,
    now_unix_seconds: u64,
    envs: &[(&str, String)],
) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_lfg"));
    command
        .args(args)
        .env("LFG_PYPI_REGISTRY_URL", registry_base_url)
        .env("LFG_NOW_UNIX_SECONDS", now_unix_seconds.to_string())
        .env("LFG_REVIEW_PROVIDER", "none");

    for (key, value) in envs {
        command.env(key, value);
    }

    command.output().expect("run lfg binary")
}

pub(crate) fn run_lfg_with_crates_io_registry_now_and_env(
    args: &[&str],
    registry_base_url: &str,
    now_unix_seconds: u64,
    envs: &[(&str, String)],
) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_lfg"));
    command
        .args(args)
        .env("LFG_CRATES_IO_REGISTRY_URL", registry_base_url)
        .env("LFG_NOW_UNIX_SECONDS", now_unix_seconds.to_string())
        .env("LFG_REVIEW_PROVIDER", "none");

    for (key, value) in envs {
        command.env(key, value);
    }

    command.output().expect("run lfg binary")
}

pub(crate) fn run_lfg_with_rubygems_registry_now_and_env(
    args: &[&str],
    registry_base_url: &str,
    now_unix_seconds: u64,
    envs: &[(&str, String)],
) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_lfg"));
    command
        .args(args)
        .env("LFG_RUBYGEMS_REGISTRY_URL", registry_base_url)
        .env("LFG_NOW_UNIX_SECONDS", now_unix_seconds.to_string())
        .env("LFG_REVIEW_PROVIDER", "none");

    for (key, value) in envs {
        command.env(key, value);
    }

    command.output().expect("run lfg binary")
}

pub(crate) fn run_program_with_registry_now_and_env(
    program: &Path,
    args: &[&str],
    registry_base_url: &str,
    now_unix_seconds: u64,
    envs: &[(&str, String)],
) -> std::process::Output {
    let mut command = Command::new(program);
    command
        .args(args)
        .env("LFG_NPM_REGISTRY_URL", registry_base_url)
        .env("LFG_NOW_UNIX_SECONDS", now_unix_seconds.to_string())
        .env("LFG_REVIEW_PROVIDER", "none");

    for (key, value) in envs {
        command.env(key, value);
    }

    command.output().expect("run lfg shim")
}
