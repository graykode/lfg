use std::env;
use std::io::{self, IsTerminal, Write};
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
use crate::core::{ReleaseReviewer, ResolvedPackageReleases, SkipReason};
use crate::evidence::{HttpArchiveFetcher, UnifiedDiffEngine};
use crate::providers::{ArchiveDiffReviewer, ProviderError, ReviewPrompt, ReviewProvider};
use crate::shims::{
    install_shim, parse_shim_command, uninstall_shim, ShimCommand, ShimCommandError, ShimSetupError,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliResponse {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AskConfirmation {
    Accepted,
    Declined,
    Unavailable,
}

pub trait AskConfirmer {
    fn confirm(&mut self, prompt: &str) -> AskConfirmation;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ManagerRunMode {
    InstallGuard,
    ReviewOnly,
}

trait ProgressReporter {
    fn report(&self, output: &str);

    fn is_immediate(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NoopProgressReporter;

impl ProgressReporter for NoopProgressReporter {
    fn report(&self, _output: &str) {}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct StdioProgressReporter;

impl ProgressReporter for StdioProgressReporter {
    fn report(&self, output: &str) {
        let mut stderr = io::stderr().lock();
        let _ = write!(stderr, "{output}").and_then(|_| stderr.flush());
    }

    fn is_immediate(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NonInteractiveAskConfirmer;

impl AskConfirmer for NonInteractiveAskConfirmer {
    fn confirm(&mut self, _prompt: &str) -> AskConfirmation {
        AskConfirmation::Unavailable
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StdioAskConfirmer;

impl AskConfirmer for StdioAskConfirmer {
    fn confirm(&mut self, prompt: &str) -> AskConfirmation {
        if !io::stdin().is_terminal() || !io::stderr().is_terminal() {
            return AskConfirmation::Unavailable;
        }

        let mut stderr = io::stderr();
        if write!(stderr, "{prompt}")
            .and_then(|_| stderr.flush())
            .is_err()
        {
            return AskConfirmation::Unavailable;
        }

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            return AskConfirmation::Unavailable;
        }

        match input.trim().to_ascii_lowercase().as_str() {
            "y" | "yes" => AskConfirmation::Accepted,
            _ => AskConfirmation::Declined,
        }
    }
}

pub fn run(args: impl IntoIterator<Item = String>) -> CliResponse {
    let mut confirmer = NonInteractiveAskConfirmer;
    let reporter = NoopProgressReporter;
    run_with_ask_confirmer(args, &mut confirmer, &reporter)
}

pub fn run_interactive(args: impl IntoIterator<Item = String>) -> CliResponse {
    let mut confirmer = StdioAskConfirmer;
    let reporter = StdioProgressReporter;
    run_with_ask_confirmer(args, &mut confirmer, &reporter)
}

fn run_with_ask_confirmer(
    args: impl IntoIterator<Item = String>,
    confirmer: &mut dyn AskConfirmer,
    reporter: &dyn ProgressReporter,
) -> CliResponse {
    let mut args = args.into_iter();
    let program = args.next().unwrap_or_default();
    let invocation_program_path = PathBuf::from(&program);

    if let Some(manager_id) = manager_id_from_program(&program) {
        return run_manager(
            manager_id,
            args.collect(),
            invocation_program_path,
            confirmer,
            reporter,
            ManagerRunMode::InstallGuard,
        );
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
            stdout: format!("packvet {}\n", env!("CARGO_PKG_VERSION")),
            stderr: String::new(),
        },
        Some(argument) if argument == "shim" => run_shim_command(args.collect()),
        Some(argument) if argument == "review" => {
            run_review_command(args.collect(), invocation_program_path, reporter)
        }
        Some(argument) => run_manager(
            &argument,
            args.collect(),
            invocation_program_path,
            confirmer,
            reporter,
            ManagerRunMode::InstallGuard,
        ),
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
    confirmer: &mut dyn AskConfirmer,
    reporter: &dyn ProgressReporter,
    mode: ManagerRunMode,
) -> CliResponse {
    let registry = match built_in_manager_adapters() {
        Ok(registry) => registry,
        Err(_) => return adapter_unavailable_response(manager_id),
    };
    let adapter = match registry.get(manager_id) {
        Ok(adapter) => adapter,
        Err(_) => return unknown_argument_response(manager_id),
    };

    if mode == ManagerRunMode::InstallGuard && bypass_requested() {
        return execute_manager_args(adapter.id(), args, invocation_program_path);
    }

    match adapter.parse_install(&args) {
        Ok(request) => evaluate_manager_request(
            adapter.as_ref(),
            request,
            invocation_program_path,
            confirmer,
            reporter,
            mode,
        ),
        Err(error) => manager_parse_error_response(manager_id, error),
    }
}

fn run_review_command(
    args: Vec<String>,
    invocation_program_path: PathBuf,
    reporter: &dyn ProgressReporter,
) -> CliResponse {
    let mut args = args.into_iter();
    let Some(manager_id) = args.next() else {
        return CliResponse {
            exit_code: 1,
            stdout: String::new(),
            stderr: "packvet: review manager is required\n".to_owned(),
        };
    };

    let mut confirmer = NonInteractiveAskConfirmer;
    run_manager(
        &manager_id,
        args.collect(),
        invocation_program_path,
        &mut confirmer,
        reporter,
        ManagerRunMode::ReviewOnly,
    )
}

fn bypass_requested() -> bool {
    matches!(
        env::var("PACKVET_BYPASS").ok().as_deref(),
        Some("1" | "true" | "yes")
    )
}

fn run_shim_command(args: Vec<String>) -> CliResponse {
    let command = match parse_shim_command(&args) {
        Ok(command) => command,
        Err(error) => return shim_command_error_response(error),
    };

    let manager_id = match &command {
        ShimCommand::Install { manager_id, .. } | ShimCommand::Uninstall { manager_id, .. } => {
            manager_id
        }
    };
    let registry = match built_in_manager_adapters() {
        Ok(registry) => registry,
        Err(_) => return adapter_unavailable_response(manager_id),
    };
    if registry.get(manager_id).is_err() {
        return unknown_argument_response(manager_id);
    }

    let packvet_executable = match env::current_exe() {
        Ok(path) => path,
        Err(error) => {
            return CliResponse {
                exit_code: Verdict::Ask.exit_code(),
                stdout: String::new(),
                stderr: format!("packvet: could not locate packvet executable: {error}\n"),
            };
        }
    };

    match command {
        ShimCommand::Install { manager_id, dir } => {
            match install_shim(&manager_id, &dir, &packvet_executable) {
                Ok(path) => CliResponse {
                    exit_code: 0,
                    stdout: format!(
                        "packvet: installed {manager_id} shim at {}\n",
                        path.display()
                    ),
                    stderr: String::new(),
                },
                Err(error) => shim_setup_error_response(error),
            }
        }
        ShimCommand::Uninstall { manager_id, dir } => {
            match uninstall_shim(&manager_id, &dir, &packvet_executable) {
                Ok(path) => CliResponse {
                    exit_code: 0,
                    stdout: format!(
                        "packvet: removed {manager_id} shim from {}\n",
                        path.display()
                    ),
                    stderr: String::new(),
                },
                Err(error) => shim_setup_error_response(error),
            }
        }
    }
}

fn shim_command_error_response(error: ShimCommandError) -> CliResponse {
    let message = match error {
        ShimCommandError::MissingAction => "packvet: shim action is required\n".to_owned(),
        ShimCommandError::MissingDir => "packvet: shim --dir is required\n".to_owned(),
        ShimCommandError::MissingManager => "packvet: shim manager is required\n".to_owned(),
        ShimCommandError::UnsupportedAction(action) => {
            format!("packvet: unsupported shim action: {action}\n")
        }
        ShimCommandError::UnknownArgument(argument) => {
            format!("packvet: unknown shim argument: {argument}\n")
        }
    };

    CliResponse {
        exit_code: 1,
        stdout: String::new(),
        stderr: message,
    }
}

fn shim_setup_error_response(error: ShimSetupError) -> CliResponse {
    let message = match error {
        ShimSetupError::ExistingPath(path) => {
            format!("packvet: shim target already exists: {}\n", path.display())
        }
        ShimSetupError::NotPackvetShim(path) => {
            format!("packvet: not a packvet shim: {}\n", path.display())
        }
        ShimSetupError::Io(error) => format!("packvet: shim setup failed: {error}\n"),
    };

    CliResponse {
        exit_code: 1,
        stdout: String::new(),
        stderr: message,
    }
}

struct ProgressReviewProvider<'a, P> {
    inner: P,
    reporter: &'a dyn ProgressReporter,
}

impl<P> ReviewProvider for ProgressReviewProvider<'_, P>
where
    P: ReviewProvider,
{
    fn id(&self) -> &'static str {
        self.inner.id()
    }

    fn review(&self, prompt: &ReviewPrompt) -> Result<String, ProviderError> {
        let provider_id = self.inner.id();
        if provider_id == "unavailable" {
            self.reporter
                .report("packvet: review provider unavailable\n");
        } else {
            self.reporter
                .report(&format!("packvet: reviewing diff with {provider_id}\n"));
        }
        self.inner.review(prompt)
    }
}

struct ProgressReleaseReviewer<'a, V> {
    inner: V,
    reporter: &'a dyn ProgressReporter,
}

impl<V> ReleaseReviewer for ProgressReleaseReviewer<'_, V>
where
    V: ReleaseReviewer,
{
    fn review(&self, releases: &ResolvedPackageReleases) -> PackageOutcome {
        self.reporter.report(&format!(
            "packvet: preparing diff for {} {} -> {}\n",
            releases.package_name, releases.previous.version, releases.target.version
        ));
        self.inner.review(releases)
    }
}

fn evaluate_manager_request(
    adapter: &dyn ManagerIntegrationAdapter,
    request: InstallRequest,
    invocation_program_path: PathBuf,
    confirmer: &mut dyn AskConfirmer,
    reporter: &dyn ProgressReporter,
    mode: ManagerRunMode,
) -> CliResponse {
    report_checking_request(adapter.id(), &request, reporter);

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

    report_resolving_targets(adapter.id(), &request, reporter);

    let provider = built_in_review_provider(&PathProgramDetector);
    let provider = ProgressReviewProvider {
        inner: provider,
        reporter,
    };
    let reviewer =
        ArchiveDiffReviewer::with_provider(HttpArchiveFetcher, UnifiedDiffEngine, provider);
    let reviewer = ProgressReleaseReviewer {
        inner: reviewer,
        reporter,
    };
    let outcomes = evaluate_install_request_with_reviewer(
        &request,
        resolver.as_ref(),
        evaluator.as_ref(),
        &reviewer,
        current_time(),
    );
    report_policy_progress_messages(&request, &outcomes, reporter);
    let verdict = aggregate_verdicts(&outcomes);
    if verdict == Verdict::Pass {
        let pass_messages = provider_pass_messages(&outcomes);
        reporter.report(&pass_messages);
        if mode == ManagerRunMode::ReviewOnly {
            let message = review_completed_message(adapter.id(), request.operation);
            if reporter.is_immediate() {
                reporter.report(&message);
                return CliResponse {
                    exit_code: 0,
                    stdout: String::new(),
                    stderr: String::new(),
                };
            }

            return CliResponse {
                exit_code: 0,
                stdout: String::new(),
                stderr: format!("{pass_messages}{message}"),
            };
        }

        report_running_manager(adapter.id(), &request, reporter);

        let executor = ProcessCommandExecutor::for_invocation(invocation_program_path);
        let mut response = execute_manager_request(adapter, &request, &executor);
        if !reporter.is_immediate() {
            response.stderr = format!("{pass_messages}{}", response.stderr);
        }
        return response;
    }

    if verdict == Verdict::Ask {
        let operation = operation_label(request.operation);
        let ask_message = ask_message(adapter.id(), operation, &outcomes);
        if mode == ManagerRunMode::ReviewOnly {
            return CliResponse {
                exit_code: Verdict::Ask.exit_code(),
                stdout: String::new(),
                stderr: ask_message,
            };
        }

        let executor = ProcessCommandExecutor::for_invocation(invocation_program_path);
        return confirm_install(adapter.id(), operation, ask_message, confirmer, || {
            report_running_manager(adapter.id(), &request, reporter);
            execute_manager_request(adapter, &request, &executor)
        });
    }

    let (exit_code, stderr) = cli_result(adapter.id(), request.operation, &outcomes, verdict);

    CliResponse {
        exit_code,
        stdout: String::new(),
        stderr,
    }
}

fn review_completed_message(manager_id: &str, operation: InstallOperation) -> String {
    let operation = operation_label(operation);

    format!("packvet: review completed for {manager_id} {operation}. install was not executed.\n")
}

fn confirm_install<F>(
    manager_id: &str,
    operation: &str,
    message: String,
    confirmer: &mut dyn AskConfirmer,
    execute: F,
) -> CliResponse
where
    F: FnOnce() -> CliResponse,
{
    let prompt = format!("{message}packvet: continue with {manager_id} {operation}? [y/N] ");

    match confirmer.confirm(&prompt) {
        AskConfirmation::Accepted => execute(),
        AskConfirmation::Declined => CliResponse {
            exit_code: Verdict::Ask.exit_code(),
            stdout: String::new(),
            stderr: "packvet: install cancelled by user.\n".to_owned(),
        },
        AskConfirmation::Unavailable => CliResponse {
            exit_code: Verdict::Ask.exit_code(),
            stdout: String::new(),
            stderr: message,
        },
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

fn report_checking_request(
    manager_id: &str,
    request: &InstallRequest,
    reporter: &dyn ProgressReporter,
) {
    reporter.report(&format!(
        "packvet: checking {}\n",
        manager_command_line(manager_id, &request.manager_args)
    ));
}

fn report_resolving_targets(
    manager_id: &str,
    request: &InstallRequest,
    reporter: &dyn ProgressReporter,
) {
    for target in &request.targets {
        reporter.report(&format!(
            "packvet: resolving {manager_id} metadata for {}\n",
            target.spec
        ));
    }
}

fn report_policy_progress_messages(
    request: &InstallRequest,
    outcomes: &[PackageOutcome],
    reporter: &dyn ProgressReporter,
) {
    for (target, outcome) in request.targets.iter().zip(outcomes) {
        if matches!(
            outcome,
            PackageOutcome::Skipped(SkipReason::OlderThanThreshold)
        ) {
            reporter.report(&format!(
                "packvet: skipped review for {}; older than configured threshold\n",
                target.spec
            ));
        }
    }
}

fn report_running_manager(
    manager_id: &str,
    request: &InstallRequest,
    reporter: &dyn ProgressReporter,
) {
    reporter.report(&format!(
        "packvet: running {}\n",
        manager_command_line(manager_id, &request.manager_args)
    ));
}

fn manager_command_line(manager_id: &str, args: &[String]) -> String {
    if args.is_empty() {
        return manager_id.to_owned();
    }

    format!("{manager_id} {}", args.join(" "))
}

fn provider_pass_messages(outcomes: &[PackageOutcome]) -> String {
    let style = OutputStyle::from_env();
    let mut message = String::new();

    for outcome in outcomes {
        let PackageOutcome::ProviderReview(review) = outcome else {
            continue;
        };

        if review.verdict != Verdict::Pass {
            continue;
        }

        let reason = review
            .reason
            .as_deref()
            .unwrap_or("provider returned pass without a reason");
        let log_path = review
            .log_path
            .as_deref()
            .map(display_log_path)
            .unwrap_or_else(|| "unavailable".to_owned());

        message.push_str(&format!(
            "packvet: review {} for {} {}\npackvet: {} {}\npackvet: {} {}\n",
            style.pass("passed"),
            review.package_name,
            review.version,
            style.label("reason:"),
            reason,
            style.label("review log:"),
            style.log_path(&log_path)
        ));
    }

    message
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct OutputStyle {
    color: bool,
}

impl OutputStyle {
    fn from_env() -> Self {
        let color = match env::var("PACKVET_COLOR").ok().as_deref() {
            Some("always") => true,
            Some("never") => false,
            Some("auto") | None => env::var_os("NO_COLOR").is_none() && io::stderr().is_terminal(),
            Some(_) => env::var_os("NO_COLOR").is_none() && io::stderr().is_terminal(),
        };

        Self { color }
    }

    fn pass(&self, value: &str) -> String {
        self.wrap(value, "\x1b[32m")
    }

    fn label(&self, value: &str) -> String {
        self.wrap(value, "\x1b[1m")
    }

    fn log_path(&self, value: &str) -> String {
        self.wrap(value, "\x1b[36;4m")
    }

    fn wrap(&self, value: &str, code: &str) -> String {
        if self.color {
            format!("{code}{value}\x1b[0m")
        } else {
            value.to_owned()
        }
    }
}

fn display_log_path(path: &Path) -> String {
    if let Some(home) = env::var_os("HOME") {
        let home = PathBuf::from(home);
        if let Ok(relative) = path.strip_prefix(&home) {
            if relative.as_os_str().is_empty() {
                return "~".to_owned();
            }

            return format!("~/{}", relative.display());
        }
    }

    path.display().to_string()
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
                "packvet: {} executable is unavailable; install is paused.\n",
                manager_id
            ),
        },
        Err(CommandExecutionError::Failed(_)) => CliResponse {
            exit_code: Verdict::Ask.exit_code(),
            stdout: String::new(),
            stderr: format!(
                "packvet: {} execution could not start; install is paused.\n",
                manager_id
            ),
        },
    }
}

fn unknown_argument_response(argument: &str) -> CliResponse {
    CliResponse {
        exit_code: 1,
        stdout: String::new(),
        stderr: format!("packvet: unknown argument: {argument}\n"),
    }
}

fn manager_parse_error_response(manager_id: &str, error: ManagerAdapterError) -> CliResponse {
    match error {
        ManagerAdapterError::MissingCommand => CliResponse {
            exit_code: 1,
            stdout: String::new(),
            stderr: format!("packvet: {manager_id} command is required\n"),
        },
        ManagerAdapterError::MissingPackage => CliResponse {
            exit_code: 1,
            stdout: String::new(),
            stderr: format!("packvet: {manager_id} install needs at least one package\n"),
        },
        ManagerAdapterError::MissingRequirementsFile => CliResponse {
            exit_code: 1,
            stdout: String::new(),
            stderr: format!("packvet: {manager_id} requirements file path is required\n"),
        },
        ManagerAdapterError::RequirementsFileUnavailable(path) => CliResponse {
            exit_code: Verdict::Ask.exit_code(),
            stdout: String::new(),
            stderr: format!(
                "packvet: {manager_id} requirements file is unavailable: {path}; install is paused.\n"
            ),
        },
        ManagerAdapterError::UnsupportedManagerOption(option) => CliResponse {
            exit_code: Verdict::Ask.exit_code(),
            stdout: String::new(),
            stderr: format!(
                "packvet: {manager_id} option cannot be reviewed safely: {option}; install is paused.\n"
            ),
        },
        ManagerAdapterError::UnsupportedRequirement(requirement) => CliResponse {
            exit_code: Verdict::Ask.exit_code(),
            stdout: String::new(),
            stderr: format!(
                "packvet: {manager_id} requirement cannot be reviewed safely: {requirement}; install is paused.\n"
            ),
        },
        ManagerAdapterError::UnsupportedCommand(command) => CliResponse {
            exit_code: 1,
            stdout: String::new(),
            stderr: format!("packvet: unsupported {manager_id} command: {command}\n"),
        },
    }
}

fn adapter_unavailable_response(manager_id: &str) -> CliResponse {
    CliResponse {
        exit_code: Verdict::Ask.exit_code(),
        stdout: String::new(),
        stderr: format!("packvet: {manager_id} adapter is unavailable; install is paused.\n"),
    }
}

fn evaluator_unavailable_response(manager_id: &str) -> CliResponse {
    CliResponse {
        exit_code: Verdict::Ask.exit_code(),
        stdout: String::new(),
        stderr: format!(
            "packvet: {manager_id} policy evaluator is unavailable; install is paused.\n"
        ),
    }
}

fn resolver_unavailable_response(manager_id: &str) -> CliResponse {
    CliResponse {
        exit_code: Verdict::Ask.exit_code(),
        stdout: String::new(),
        stderr: format!("packvet: {manager_id} resolver is unavailable; install is paused.\n"),
    }
}

fn policy_config_error_response(manager_id: &str) -> CliResponse {
    CliResponse {
        exit_code: Verdict::Ask.exit_code(),
        stdout: String::new(),
        stderr: format!(
            "packvet: {manager_id} review policy configuration is invalid; install is paused.\n"
        ),
    }
}

fn current_time() -> SystemTime {
    env::var("PACKVET_NOW_UNIX_SECONDS")
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
                "packvet: {manager_id} {operation} reached pass verdict without execution; install is paused.\n"
            ),
        ),
        Verdict::Ask => (
            Verdict::Ask.exit_code(),
            ask_message(manager_id, operation, outcomes),
        ),
        Verdict::Block => (
            Verdict::Block.exit_code(),
            format!("packvet: {manager_id} {operation} was blocked by provider review.\n"),
        ),
    }
}

fn operation_label(operation: InstallOperation) -> &'static str {
    match operation {
        InstallOperation::Add => "add",
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
            "packvet: review required for {manager_id} {operation}, but archive diff review is not wired yet. install is paused.\n"
        );
    }

    if outcomes.iter().any(|outcome| {
        matches!(
            outcome,
            PackageOutcome::ReviewUnavailable(ReviewUnavailableReason::RegistryFailure)
        )
    }) {
        return format!(
            "packvet: {manager_id} registry metadata is unavailable; install is paused.\n"
        );
    }

    if outcomes.iter().any(|outcome| {
        matches!(
            outcome,
            PackageOutcome::ReviewUnavailable(ReviewUnavailableReason::ProviderFailure)
        )
    }) {
        return format!(
            "packvet: review required for {manager_id} {operation}, but provider review is not wired yet. install is paused.\n"
        );
    }

    if outcomes.iter().any(|outcome| {
        matches!(
            outcome,
            PackageOutcome::PolicyAsk(AskReason::MissingPreviousRelease)
        )
    }) {
        return format!(
            "packvet: {manager_id} package has no previous release to diff; install is paused.\n"
        );
    }

    if outcomes.iter().any(|outcome| {
        matches!(
            outcome,
            PackageOutcome::PolicyAsk(AskReason::MissingTargetPublishTime)
        )
    }) {
        return format!(
            "packvet: {manager_id} package publish time is missing or invalid; install is paused.\n"
        );
    }

    format!("packvet: {manager_id} review could not complete safely; install is paused.\n")
}

fn help_text() -> String {
    "\
packvet is a local pre-install guard for package managers.

Usage: packvet [OPTIONS] [MANAGER] [ARGS]
       packvet review <MANAGER> [ARGS]
       packvet shim install --dir <DIR> <MANAGER>
       packvet shim uninstall --dir <DIR> <MANAGER>

Options:
  -h, --help       Print help
  -V, --version    Print version

Examples:
  packvet review npm install <package>
  packvet cargo add <crate>
  packvet gem install <gem>
  packvet npm install <package>
  packvet pnpm add <package>
  packvet yarn add <package>
  packvet pip install -r requirements.txt
  packvet uv add <package>
  packvet shim install --dir ~/.local/bin pnpm
"
    .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_args_returns_ask_without_output() {
        let response = run(["packvet".to_owned()]);

        assert_eq!(response.exit_code, 20);
        assert!(response.stdout.is_empty());
        assert!(response.stderr.is_empty());
    }

    #[test]
    fn help_returns_success_with_usage() {
        let response = run(["packvet".to_owned(), "--help".to_owned()]);

        assert_eq!(response.exit_code, 0);
        assert!(response.stdout.contains("Usage: packvet"));
        assert!(response.stderr.is_empty());
    }

    #[test]
    fn unknown_arg_returns_cli_misuse() {
        let response = run(["packvet".to_owned(), "--bad".to_owned()]);

        assert_eq!(response.exit_code, 1);
        assert!(response.stdout.is_empty());
        assert_eq!(response.stderr, "packvet: unknown argument: --bad\n");
    }

    #[test]
    fn shim_invocation_uses_program_name_as_manager() {
        let response = run(["/tmp/npm".to_owned(), "install".to_owned()]);

        assert_eq!(response.exit_code, 1);
        assert!(response.stdout.is_empty());
        assert_eq!(
            response.stderr,
            "packvet: npm install needs at least one package\n"
        );
    }

    #[test]
    fn unsupported_manager_option_pauses_install() {
        let response = run([
            "packvet".to_owned(),
            "pip".to_owned(),
            "install".to_owned(),
            "--index-url".to_owned(),
            "https://example.invalid/simple".to_owned(),
            "requests".to_owned(),
        ]);

        assert_eq!(response.exit_code, 20);
        assert!(response.stdout.is_empty());
        assert_eq!(
            response.stderr,
            "packvet: pip option cannot be reviewed safely: --index-url; install is paused.\n"
        );
    }

    #[derive(Debug, Clone)]
    struct StaticAskConfirmer {
        decision: AskConfirmation,
        prompts: Vec<String>,
    }

    impl AskConfirmer for StaticAskConfirmer {
        fn confirm(&mut self, prompt: &str) -> AskConfirmation {
            self.prompts.push(prompt.to_owned());
            self.decision
        }
    }

    #[test]
    fn accepted_ask_confirmation_executes_manager() {
        let mut confirmer = StaticAskConfirmer {
            decision: AskConfirmation::Accepted,
            prompts: Vec::new(),
        };

        let response = confirm_install(
            "npm",
            "install",
            "packvet: review could not complete safely; install is paused.\n".to_owned(),
            &mut confirmer,
            || CliResponse {
                exit_code: 0,
                stdout: "ran npm\n".to_owned(),
                stderr: String::new(),
            },
        );

        assert_eq!(response.exit_code, 0);
        assert_eq!(response.stdout, "ran npm\n");
        assert_eq!(
            confirmer.prompts,
            vec![
                "packvet: review could not complete safely; install is paused.\npackvet: continue with npm install? [y/N] "
            ]
        );
    }

    #[test]
    fn declined_ask_confirmation_stays_paused() {
        let mut confirmer = StaticAskConfirmer {
            decision: AskConfirmation::Declined,
            prompts: Vec::new(),
        };

        let response = confirm_install(
            "npm",
            "install",
            "packvet: review could not complete safely; install is paused.\n".to_owned(),
            &mut confirmer,
            || CliResponse {
                exit_code: 0,
                stdout: "ran npm\n".to_owned(),
                stderr: String::new(),
            },
        );

        assert_eq!(response.exit_code, 20);
        assert!(response.stdout.is_empty());
        assert_eq!(response.stderr, "packvet: install cancelled by user.\n");
    }

    #[test]
    fn unavailable_ask_confirmation_keeps_non_interactive_ask_response() {
        let mut confirmer = StaticAskConfirmer {
            decision: AskConfirmation::Unavailable,
            prompts: Vec::new(),
        };

        let response = confirm_install(
            "npm",
            "install",
            "packvet: review could not complete safely; install is paused.\n".to_owned(),
            &mut confirmer,
            || CliResponse {
                exit_code: 0,
                stdout: "ran npm\n".to_owned(),
                stderr: String::new(),
            },
        );

        assert_eq!(response.exit_code, 20);
        assert!(response.stdout.is_empty());
        assert_eq!(
            response.stderr,
            "packvet: review could not complete safely; install is paused.\n"
        );
    }
}
