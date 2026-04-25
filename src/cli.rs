use crate::npm::{parse_npm_install, NpmParseError};
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
    match parse_npm_install(&args) {
        Ok(_) => CliResponse {
            exit_code: Verdict::Ask.exit_code(),
            stdout: String::new(),
            stderr: "lfg: npm install review is not wired yet, so install is paused.\n".to_owned(),
        },
        Err(NpmParseError::MissingCommand) => CliResponse {
            exit_code: 1,
            stdout: String::new(),
            stderr: "lfg: npm command is required\n".to_owned(),
        },
        Err(NpmParseError::MissingPackage) => CliResponse {
            exit_code: 1,
            stdout: String::new(),
            stderr: "lfg: npm install needs at least one package\n".to_owned(),
        },
        Err(NpmParseError::UnsupportedCommand(command)) => CliResponse {
            exit_code: 1,
            stdout: String::new(),
            stderr: format!("lfg: unsupported npm command: {command}\n"),
        },
    }
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
