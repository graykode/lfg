use std::env;
use std::time::{Duration, SystemTime};

use crate::adapters::{ManagerAdapterError, ManagerIntegrationAdapter};
use crate::npm::NpmManagerAdapter;
use crate::npm_registry::{NpmHttpPackumentClient, NpmRegistryResolver};
use crate::npm_review::evaluate_npm_install_request;
use crate::orchestrator::{aggregate_verdicts, PackageOutcome, ReviewUnavailableReason};
use crate::policy::{AskReason, ReviewPolicy};
use crate::verdict::Verdict;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliResponse {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

pub fn run(args: impl IntoIterator<Item = String>) -> CliResponse {
    let mut args = args.into_iter();
    let _program = args.next();

    match args.next() {
        None => CliResponse {
            exit_code: Verdict::Ask.exit_code(),
            stdout: String::new(),
            stderr: String::new(),
        },
        Some(argument) if argument == "--help" || argument == "-h" => CliResponse {
            exit_code: 0,
            stdout: help_text(),
            stderr: String::new(),
        },
        Some(argument) if argument == "--version" || argument == "-V" => CliResponse {
            exit_code: 0,
            stdout: format!("lfg {}\n", env!("CARGO_PKG_VERSION")),
            stderr: String::new(),
        },
        Some(argument) if argument == "npm" => run_npm(args.collect()),
        Some(argument) => CliResponse {
            exit_code: 1,
            stdout: String::new(),
            stderr: format!("lfg: unknown argument: {argument}\n"),
        },
    }
}

fn run_npm(args: Vec<String>) -> CliResponse {
    match NpmManagerAdapter.parse_install(&args) {
        Ok(request) => evaluate_npm_request(request),
        Err(ManagerAdapterError::MissingCommand) => CliResponse {
            exit_code: 1,
            stdout: String::new(),
            stderr: "lfg: npm command is required\n".to_owned(),
        },
        Err(ManagerAdapterError::MissingPackage) => CliResponse {
            exit_code: 1,
            stdout: String::new(),
            stderr: "lfg: npm install needs at least one package\n".to_owned(),
        },
        Err(ManagerAdapterError::UnsupportedCommand(command)) => CliResponse {
            exit_code: 1,
            stdout: String::new(),
            stderr: format!("lfg: unsupported npm command: {command}\n"),
        },
    }
}

fn evaluate_npm_request(request: crate::install_request::InstallRequest) -> CliResponse {
    let resolver = NpmRegistryResolver::new(NpmHttpPackumentClient::new(npm_registry_base_url()));
    let outcomes = evaluate_npm_install_request(
        &request,
        &resolver,
        &ReviewPolicy::default(),
        current_time(),
    );
    let verdict = aggregate_verdicts(&outcomes);
    let (exit_code, stderr) = npm_cli_result(&outcomes, verdict);

    CliResponse {
        exit_code,
        stdout: String::new(),
        stderr,
    }
}

fn npm_registry_base_url() -> String {
    env::var("LFG_NPM_REGISTRY_URL").unwrap_or_else(|_| "https://registry.npmjs.org".to_owned())
}

fn current_time() -> SystemTime {
    env::var("LFG_NOW_UNIX_SECONDS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .map(|seconds| SystemTime::UNIX_EPOCH + Duration::from_secs(seconds))
        .unwrap_or_else(SystemTime::now)
}

fn npm_cli_result(outcomes: &[PackageOutcome], verdict: Verdict) -> (i32, String) {
    match verdict {
        Verdict::Pass => (
            Verdict::Ask.exit_code(),
            "lfg: npm review is not required by policy, but npm execution is not wired yet. install is paused.\n"
                .to_owned(),
        ),
        Verdict::Ask => (Verdict::Ask.exit_code(), npm_ask_message(outcomes)),
        Verdict::Block => (
            Verdict::Block.exit_code(),
            "lfg: npm install was blocked by provider review.\n".to_owned(),
        ),
    }
}

fn npm_ask_message(outcomes: &[PackageOutcome]) -> String {
    if outcomes.iter().any(|outcome| {
        matches!(
            outcome,
            PackageOutcome::ReviewUnavailable(ReviewUnavailableReason::DiffFailure)
        )
    }) {
        return "lfg: review required for npm install, but archive diff review is not wired yet. install is paused.\n"
            .to_owned();
    }

    if outcomes.iter().any(|outcome| {
        matches!(
            outcome,
            PackageOutcome::ReviewUnavailable(ReviewUnavailableReason::RegistryFailure)
        )
    }) {
        return "lfg: npm registry metadata is unavailable; install is paused.\n".to_owned();
    }

    if outcomes.iter().any(|outcome| {
        matches!(
            outcome,
            PackageOutcome::PolicyAsk(AskReason::MissingPreviousRelease)
        )
    }) {
        return "lfg: npm package has no previous release to diff; install is paused.\n".to_owned();
    }

    if outcomes.iter().any(|outcome| {
        matches!(
            outcome,
            PackageOutcome::PolicyAsk(AskReason::MissingTargetPublishTime)
        )
    }) {
        return "lfg: npm package publish time is missing or invalid; install is paused.\n"
            .to_owned();
    }

    "lfg: npm review could not complete safely; install is paused.\n".to_owned()
}

fn help_text() -> String {
    "\
lfg is a local pre-install guard for package managers.

Usage: lfg [OPTIONS] [MANAGER] [ARGS]

Options:
  -h, --help       Print help
  -V, --version    Print version

Examples:
  lfg npm install <package>
"
    .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_args_returns_ask_without_output() {
        let response = run(["lfg".to_owned()]);

        assert_eq!(response.exit_code, 20);
        assert!(response.stdout.is_empty());
        assert!(response.stderr.is_empty());
    }

    #[test]
    fn help_returns_success_with_usage() {
        let response = run(["lfg".to_owned(), "--help".to_owned()]);

        assert_eq!(response.exit_code, 0);
        assert!(response.stdout.contains("Usage: lfg"));
        assert!(response.stderr.is_empty());
    }

    #[test]
    fn unknown_arg_returns_cli_misuse() {
        let response = run(["lfg".to_owned(), "--bad".to_owned()]);

        assert_eq!(response.exit_code, 1);
        assert!(response.stdout.is_empty());
        assert_eq!(response.stderr, "lfg: unknown argument: --bad\n");
    }
}
