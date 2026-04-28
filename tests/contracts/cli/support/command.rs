use std::process::Command;

pub(crate) fn run_packvet(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_packvet"))
        .args(args)
        .output()
        .expect("run packvet binary")
}

pub(crate) fn run_packvet_with_registry_and_now(
    args: &[&str],
    registry_base_url: &str,
    now_unix_seconds: u64,
) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_packvet"))
        .args(args)
        .env("PACKVET_NPM_REGISTRY_URL", registry_base_url)
        .env("PACKVET_NOW_UNIX_SECONDS", now_unix_seconds.to_string())
        .env("PACKVET_REVIEW_PROVIDER", "none")
        .output()
        .expect("run packvet binary")
}

pub(crate) fn run_packvet_with_registry_now_and_env(
    args: &[&str],
    registry_base_url: &str,
    now_unix_seconds: u64,
    envs: &[(&str, String)],
) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_packvet"));
    command
        .args(args)
        .env("PACKVET_NPM_REGISTRY_URL", registry_base_url)
        .env("PACKVET_NOW_UNIX_SECONDS", now_unix_seconds.to_string())
        .env("PACKVET_REVIEW_PROVIDER", "none");

    for (key, value) in envs {
        command.env(key, value);
    }

    command.output().expect("run packvet binary")
}

pub(crate) fn run_packvet_with_pypi_registry_now_and_env(
    args: &[&str],
    registry_base_url: &str,
    now_unix_seconds: u64,
    envs: &[(&str, String)],
) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_packvet"));
    command
        .args(args)
        .env("PACKVET_PYPI_REGISTRY_URL", registry_base_url)
        .env("PACKVET_NOW_UNIX_SECONDS", now_unix_seconds.to_string())
        .env("PACKVET_REVIEW_PROVIDER", "none");

    for (key, value) in envs {
        command.env(key, value);
    }

    command.output().expect("run packvet binary")
}

pub(crate) fn run_packvet_with_crates_io_registry_now_and_env(
    args: &[&str],
    registry_base_url: &str,
    now_unix_seconds: u64,
    envs: &[(&str, String)],
) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_packvet"));
    command
        .args(args)
        .env("PACKVET_CRATES_IO_REGISTRY_URL", registry_base_url)
        .env("PACKVET_NOW_UNIX_SECONDS", now_unix_seconds.to_string())
        .env("PACKVET_REVIEW_PROVIDER", "none");

    for (key, value) in envs {
        command.env(key, value);
    }

    command.output().expect("run packvet binary")
}

pub(crate) fn run_packvet_with_rubygems_registry_now_and_env(
    args: &[&str],
    registry_base_url: &str,
    now_unix_seconds: u64,
    envs: &[(&str, String)],
) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_packvet"));
    command
        .args(args)
        .env("PACKVET_RUBYGEMS_REGISTRY_URL", registry_base_url)
        .env("PACKVET_NOW_UNIX_SECONDS", now_unix_seconds.to_string())
        .env("PACKVET_REVIEW_PROVIDER", "none");

    for (key, value) in envs {
        command.env(key, value);
    }

    command.output().expect("run packvet binary")
}
