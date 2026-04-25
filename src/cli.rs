use std::env;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use crate::builtins::{
    built_in_manager_adapters, built_in_release_decision_evaluators, built_in_release_resolvers,
    built_in_review_provider, AdapterConfig, PathProgramDetector, PolicyConfig,
};
use crate::core::Verdict;
use crate::core::{aggregate_verdicts, PackageOutcome, ReviewUnavailableReason};
use crate::core::{evaluate_install_request_with_reviewer, AskReason};
use crate::core::{CommandExecutionError, CommandExecutor, ProcessCommandExecutor};
use crate::core::{
    InstallOperation, InstallRequest, ManagerAdapterError, ManagerIntegrationAdapter, RealCommand,
};
use crate::evidence::{HttpArchiveFetcher, UnifiedDiffEngine};
use crate::providers::ArchiveDiffReviewer;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliResponse {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

pub fn run(args: impl IntoIterator<Item = String>) -> CliResponse {
    let mut args = args.into_iter();
    let program = args.next().unwrap_or_default();
    let invocation_program_path = PathBuf::from(&program);

    if let Some(manager_id) = manager_id_from_program(&program) {
        return run_manager(manager_id, args.collect(), invocation_program_path);
    }

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
        Some(argument) => run_manager(&argument, args.collect(), invocation_program_path),
    }
}

fn manager_id_from_program(program: &str) -> Option<&str> {
    let program_name = Path::new(program).file_name()?.to_str()?;
    let registry = built_in_manager_adapters().ok()?;

    registry.get(program_name).ok()?;

    Some(program_name)
}

fn run_manager(
    manager_id: &str,
    args: Vec<String>,
    invocation_program_path: PathBuf,
) -> CliResponse {
    let registry = match built_in_manager_adapters() {
        Ok(registry) => registry,
        Err(_) => return adapter_unavailable_response(manager_id),
    };
    let adapter = match registry.get(manager_id) {
        Ok(adapter) => adapter,
        Err(_) => return unknown_argument_response(manager_id),
    };

    if bypass_requested() {
        return execute_manager_args(adapter.id(), args, invocation_program_path);
    }

    match adapter.parse_install(&args) {
        Ok(request) => evaluate_manager_request(adapter.as_ref(), request, invocation_program_path),
        Err(error) => manager_parse_error_response(manager_id, error),
    }
}

fn bypass_requested() -> bool {
    matches!(
        env::var("LFG_BYPASS").ok().as_deref(),
        Some("1" | "true" | "yes")
    )
}

fn evaluate_manager_request(
    adapter: &dyn ManagerIntegrationAdapter,
    request: InstallRequest,
    invocation_program_path: PathBuf,
) -> CliResponse {
    let resolver_registry = match built_in_release_resolvers(AdapterConfig::from_env()) {
        Ok(registry) => registry,
        Err(_) => return resolver_unavailable_response(adapter.id()),
    };
    let resolver = match resolver_registry.get(adapter.release_resolver_id()) {
        Ok(resolver) => resolver,
        Err(_) => return resolver_unavailable_response(adapter.id()),
    };

    let policy = match PolicyConfig::from_env() {
        Ok(config) => config.review_policy(),
        Err(_) => return policy_config_error_response(adapter.id()),
    };
    let evaluator_registry = match built_in_release_decision_evaluators(&policy) {
        Ok(registry) => registry,
        Err(_) => return evaluator_unavailable_response(adapter.id()),
    };
    let evaluator = match evaluator_registry.get(adapter.release_decision_evaluator_id()) {
        Ok(evaluator) => evaluator,
        Err(_) => return evaluator_unavailable_response(adapter.id()),
    };

    let provider = built_in_review_provider(&PathProgramDetector);
    let reviewer =
        ArchiveDiffReviewer::with_provider(HttpArchiveFetcher, UnifiedDiffEngine, provider);
    let outcomes = evaluate_install_request_with_reviewer(
        &request,
        resolver.as_ref(),
        evaluator.as_ref(),
        &reviewer,
        current_time(),
    );
    let verdict = aggregate_verdicts(&outcomes);
    if verdict == Verdict::Pass {
        let executor = ProcessCommandExecutor::for_invocation(invocation_program_path);
        return execute_manager_request(adapter, &request, &executor);
    }

    let (exit_code, stderr) = cli_result(adapter.id(), request.operation, &outcomes, verdict);

    CliResponse {
        exit_code,
        stdout: String::new(),
        stderr,
    }
}

fn execute_manager_request(
    adapter: &dyn ManagerIntegrationAdapter,
    request: &InstallRequest,
    executor: &dyn CommandExecutor,
) -> CliResponse {
    let command = adapter.real_command(request);

    execute_real_command(adapter.id(), command, executor)
}

fn execute_manager_args(
    manager_id: &str,
    args: Vec<String>,
    invocation_program_path: PathBuf,
) -> CliResponse {
    let executor = ProcessCommandExecutor::for_invocation(invocation_program_path);
    execute_real_command(
        manager_id,
        RealCommand {
            program: manager_id.to_owned(),
            args,
        },
        &executor,
    )
}

fn execute_real_command(
    manager_id: &str,
    command: RealCommand,
    executor: &dyn CommandExecutor,
) -> CliResponse {
    match executor.execute(&command) {
        Ok(output) => CliResponse {
            exit_code: output.exit_code,
            stdout: output.stdout,
            stderr: output.stderr,
        },
        Err(CommandExecutionError::Unavailable(_)) => CliResponse {
            exit_code: Verdict::Ask.exit_code(),
            stdout: String::new(),
            stderr: format!(
                "lfg: {} executable is unavailable; install is paused.\n",
                manager_id
            ),
        },
        Err(CommandExecutionError::Failed(_)) => CliResponse {
            exit_code: Verdict::Ask.exit_code(),
            stdout: String::new(),
            stderr: format!(
                "lfg: {} execution could not start; install is paused.\n",
                manager_id
            ),
        },
    }
}

fn unknown_argument_response(argument: &str) -> CliResponse {
    CliResponse {
        exit_code: 1,
        stdout: String::new(),
        stderr: format!("lfg: unknown argument: {argument}\n"),
    }
}

fn manager_parse_error_response(manager_id: &str, error: ManagerAdapterError) -> CliResponse {
    match error {
        ManagerAdapterError::MissingCommand => CliResponse {
            exit_code: 1,
            stdout: String::new(),
            stderr: format!("lfg: {manager_id} command is required\n"),
        },
        ManagerAdapterError::MissingPackage => CliResponse {
            exit_code: 1,
            stdout: String::new(),
            stderr: format!("lfg: {manager_id} install needs at least one package\n"),
        },
        ManagerAdapterError::UnsupportedCommand(command) => CliResponse {
            exit_code: 1,
            stdout: String::new(),
            stderr: format!("lfg: unsupported {manager_id} command: {command}\n"),
        },
    }
}

fn adapter_unavailable_response(manager_id: &str) -> CliResponse {
    CliResponse {
        exit_code: Verdict::Ask.exit_code(),
        stdout: String::new(),
        stderr: format!("lfg: {manager_id} adapter is unavailable; install is paused.\n"),
    }
}

fn evaluator_unavailable_response(manager_id: &str) -> CliResponse {
    CliResponse {
        exit_code: Verdict::Ask.exit_code(),
        stdout: String::new(),
        stderr: format!("lfg: {manager_id} policy evaluator is unavailable; install is paused.\n"),
    }
}

fn resolver_unavailable_response(manager_id: &str) -> CliResponse {
    CliResponse {
        exit_code: Verdict::Ask.exit_code(),
        stdout: String::new(),
        stderr: format!("lfg: {manager_id} resolver is unavailable; install is paused.\n"),
    }
}

fn policy_config_error_response(manager_id: &str) -> CliResponse {
    CliResponse {
        exit_code: Verdict::Ask.exit_code(),
        stdout: String::new(),
        stderr: format!(
            "lfg: {manager_id} review policy configuration is invalid; install is paused.\n"
        ),
    }
}

fn current_time() -> SystemTime {
    env::var("LFG_NOW_UNIX_SECONDS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .map(|seconds| SystemTime::UNIX_EPOCH + Duration::from_secs(seconds))
        .unwrap_or_else(SystemTime::now)
}

fn cli_result(
    manager_id: &str,
    operation: InstallOperation,
    outcomes: &[PackageOutcome],
    verdict: Verdict,
) -> (i32, String) {
    let operation = operation_label(operation);

    match verdict {
        Verdict::Pass => (
            Verdict::Ask.exit_code(),
            format!(
                "lfg: {manager_id} {operation} reached pass verdict without execution; install is paused.\n"
            ),
        ),
        Verdict::Ask => (
            Verdict::Ask.exit_code(),
            ask_message(manager_id, operation, outcomes),
        ),
        Verdict::Block => (
            Verdict::Block.exit_code(),
            format!("lfg: {manager_id} {operation} was blocked by provider review.\n"),
        ),
    }
}

fn operation_label(operation: InstallOperation) -> &'static str {
    match operation {
        InstallOperation::Install => "install",
    }
}

fn ask_message(manager_id: &str, operation: &str, outcomes: &[PackageOutcome]) -> String {
    if outcomes.iter().any(|outcome| {
        matches!(
            outcome,
            PackageOutcome::ReviewUnavailable(ReviewUnavailableReason::DiffFailure)
        )
    }) {
        return format!(
            "lfg: review required for {manager_id} {operation}, but archive diff review is not wired yet. install is paused.\n"
        );
    }

    if outcomes.iter().any(|outcome| {
        matches!(
            outcome,
            PackageOutcome::ReviewUnavailable(ReviewUnavailableReason::RegistryFailure)
        )
    }) {
        return format!("lfg: {manager_id} registry metadata is unavailable; install is paused.\n");
    }

    if outcomes.iter().any(|outcome| {
        matches!(
            outcome,
            PackageOutcome::ReviewUnavailable(ReviewUnavailableReason::ProviderFailure)
        )
    }) {
        return format!(
            "lfg: review required for {manager_id} {operation}, but provider review is not wired yet. install is paused.\n"
        );
    }

    if outcomes.iter().any(|outcome| {
        matches!(
            outcome,
            PackageOutcome::PolicyAsk(AskReason::MissingPreviousRelease)
        )
    }) {
        return format!(
            "lfg: {manager_id} package has no previous release to diff; install is paused.\n"
        );
    }

    if outcomes.iter().any(|outcome| {
        matches!(
            outcome,
            PackageOutcome::PolicyAsk(AskReason::MissingTargetPublishTime)
        )
    }) {
        return format!(
            "lfg: {manager_id} package publish time is missing or invalid; install is paused.\n"
        );
    }

    format!("lfg: {manager_id} review could not complete safely; install is paused.\n")
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

    #[test]
    fn shim_invocation_uses_program_name_as_manager() {
        let response = run(["/tmp/npm".to_owned(), "install".to_owned()]);

        assert_eq!(response.exit_code, 1);
        assert!(response.stdout.is_empty());
        assert_eq!(
            response.stderr,
            "lfg: npm install needs at least one package\n"
        );
    }
}
